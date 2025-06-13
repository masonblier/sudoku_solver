//! GPU batch solver
use std::num::NonZeroU32;
use wgpu::{util::DeviceExt, Features};
use crate::sudoku_board::{SudokuBoard};

const BOARD_BUFFER_SIZE: u64 = 81 * 4; // 81 * size_of::<u32>
const NUM_BUFFERS: u32 = 1;
const DISPATCH_COUNT: u32 = 9 * 9 * 9 * 9 * NUM_BUFFERS;


// execute batch solve on gpu
pub async fn gpu_solve_boards(boards: &Vec<SudokuBoard>) -> Vec<Option<SudokuBoard>> {

    // batch into groups for buffer limits
    let batches = boards.chunks(NUM_BUFFERS as usize);

    let mut results: Vec<Option<SudokuBoard>> = vec![];
    for batch in batches {
        let batch_values = batch.iter().map(|board| -> [u32; 81] {
            bytemuck::cast(board.values)
        }).collect::<Vec<[u32; 81]>>();
        for result in execute_gpu(&batch_values).await {
            results.push(if result.iter().any(|v| *v == 0) {
                None
            } else {
                Some(SudokuBoard { values: bytemuck::cast(result) })
            });
        }
    }

    results
}

// setup and execute gpu
pub async fn execute_gpu(batch_values: &Vec<[u32; 81]>) -> Vec<[u32; 81]> {
    // setup gpu adapter
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            // These features are required to use `binding_array` in your wgsl.
            // Without them your shader may fail to compile.
            required_features: Features::BUFFER_BINDING_ARRAY
                | Features::STORAGE_RESOURCE_BINDING_ARRAY
                | Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
            memory_hints: wgpu::MemoryHints::Performance,
            required_limits: wgpu::Limits {
                max_buffer_size: BOARD_BUFFER_SIZE,
                max_binding_array_elements_per_shader_stage: NUM_BUFFERS,
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap();

    // execute gpu commands
    execute_gpu_inner(&device, &queue, batch_values).await
}

// async execute gpu commands
pub async fn execute_gpu_inner(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    batch_values: &Vec<[u32; 81]>,
) -> Vec<[u32; 81]> {
    // setup pipeline
    let (dest_buffers, source_buffers, bind_group, compute_pipeline) = setup(device, batch_values);

    // setup compute command
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("compute pass descriptor"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, Some(&bind_group), &[]);

        cpass.dispatch_workgroups(DISPATCH_COUNT as u32, 1, 1);
    }

    for (source_buffer, dest_buffer) in source_buffers.iter().zip(dest_buffers.iter()) {
        let stg_size = dest_buffer.size();

        encoder.copy_buffer_to_buffer(
            source_buffer, // Source buffer
            0,
            dest_buffer, // Destination buffer
            0,
            stg_size,
        );
    }

    // submit commands
    queue.submit(Some(encoder.finish()));

    // setup read slices
    for dest_buffer in &dest_buffers {
        let slice = dest_buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
    }

    // wait for gpu
    device.poll(wgpu::PollType::Wait).unwrap();

    // read data from staging buffers
    let mut data: Vec<[u32; 81]> = Vec::new();
    for dest_buffer in &dest_buffers {
        let slice = dest_buffer.slice(..);
        let mapped = slice.get_mapped_range();
        data.push(bytemuck::cast_slice(&mapped)[0..81].try_into().unwrap());
        drop(mapped);
        dest_buffer.unmap();
    }

    data
}

fn setup(
    device: &wgpu::Device,
    batch_values: &Vec<[u32; 81]>,
) -> (
    Vec<wgpu::Buffer>,
    Vec<wgpu::Buffer>,
    wgpu::BindGroup,
    wgpu::ComputePipeline,
) {
    let cs_module = device.create_shader_module(wgpu::include_wgsl!("gpu_solver.wgsl"));

    let dest_buffers = create_dest_buffers(device);
    let source_buffers = create_source_buffers(device, batch_values);

    let (bind_group_layout, bind_group) = setup_binds(&source_buffers, device);

    let compute_pipeline = setup_pipeline(device, bind_group_layout, cs_module);
    (
        dest_buffers,
        source_buffers,
        bind_group,
        compute_pipeline,
    )
}

fn setup_pipeline(
    device: &wgpu::Device,
    bind_group_layout: wgpu::BindGroupLayout,
    cs_module: wgpu::ShaderModule,
) -> wgpu::ComputePipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Compute Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &cs_module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    })
}

fn setup_binds(
    source_buffers: &[wgpu::Buffer],
    device: &wgpu::Device,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let bind_group_entries: Vec<wgpu::BindGroupEntry> = source_buffers
        .iter()
        .enumerate()
        .map(|(bind_idx, buffer)| wgpu::BindGroupEntry {
            binding: bind_idx as u32,
            resource: buffer.as_entire_binding(),
        })
        .collect();

    let bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry> = (0..source_buffers.len())
        .map(|bind_idx| wgpu::BindGroupLayoutEntry {
            binding: bind_idx as u32,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: Some(NonZeroU32::new(1).unwrap()),
        })
        .collect();

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Custom Storage Bind Group Layout"),
        entries: &bind_group_layout_entries,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Combined Storage Bind Group"),
        layout: &bind_group_layout,
        entries: &bind_group_entries,
    });

    (bind_group_layout, bind_group)
}

fn create_source_buffers(device: &wgpu::Device, batch_values: &Vec<[u32; 81]>) -> Vec<wgpu::Buffer> {
    (0..NUM_BUFFERS)
        .map(|e| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Source Buffer-{}", e)),
                contents: bytemuck::cast_slice(&batch_values[e as usize]),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            })
        })
        .collect()
}

fn create_dest_buffers(device: &wgpu::Device) -> Vec<wgpu::Buffer> {
    (0..NUM_BUFFERS)
        .map(|e| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Dest buffer-{}", e)),
                size: BOARD_BUFFER_SIZE,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        })
        .collect()
}
