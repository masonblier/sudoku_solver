// must match gpu_solver NUM_BUFFERS
const NUM_BUFFERS: u32 = 1u;

// `binding_array` requires a custom struct
struct SimpleArray {
    inner: array<u32>
}
// for try_insert_cell result handling
struct TryInsertResult {
    mg: array<u32, 81>,
    success: bool,
}

@group(0) @binding(0)
var<storage, read_write> storage_array: binding_array<SimpleArray, NUM_BUFFERS>;


@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // batch across buffers
    let buffer_index = global_id.x / 6561;
    // dispatch values to evaluate
    let first_value = 1 + (global_id.x % 6561) / 729;
    let second_value = 1 + (global_id.x % 729) / 81;
    let third_value = 1 + (global_id.x % 81) / 9;
    let last_value = 1 + global_id.x % 9;

    // find first cell to process
    var first_index = 0u;
    for (var i = 0u; i < 81u; i++) {
        if (storage_array[buffer_index].inner[i] == 0) {
            first_index = i;
            break;
        }
    }
    // find second cell to process
    var second_index = 0u;
    for (var i = 1u + first_index; i < 81u; i++) {
        if (storage_array[buffer_index].inner[i] == 0) {
            second_index = i;
            break;
        }
    }
    // find third cell to process
    var third_index = 0u;
    for (var i = 1u + second_index; i < 81u; i++) {
        if (storage_array[buffer_index].inner[i] == 0) {
            third_index = i;
            break;
        }
    }
    // find last cell to process
    var last_index = 0u;
    for (var i = 1u + third_index; i < 81u; i++) {
        if (storage_array[buffer_index].inner[i] == 0) {
            last_index = i;
            break;
        }
    }

    // build move groups array
    var move_group = array<u32, 81>();
    for (var i = 0u; i < 81u; i++) {
        let val = storage_array[buffer_index].inner[i];
        if (val > 0) {
            // insert only fixed value
            move_group[i] = 1u << (val - 1);
        } else {
            // insert all possible moves
            for (var m = 0u; m < 9u; m++) {
                move_group[i] += 1u << m;
            }
        }
    }

    // try to insert first value
    let r1 = try_insert_cell(move_group, first_index, first_value);
    if (!r1.success) { return; }
    move_group = r1.mg;
    // try to insert second value
    let r2 = try_insert_cell(move_group, second_index, second_value);
    if (!r2.success) { return; }
    move_group = r2.mg;
    // try to insert third value
    let r3 = try_insert_cell(move_group, third_index, third_value);
    if (!r3.success) { return; }
    move_group = r3.mg;
    // try to insert last value
    let rl = try_insert_cell(move_group, last_index, last_value);
    if (!rl.success) { return; }
    move_group = rl.mg;

    // depth-first insert attempts
    // track depth history
    var depth_index = 0u;
    var depth_stack = array<u32, 81>();
    var depth_mgs = array<array<u32, 81>, 81>();

    // copy initial move group data
    for (var i = 0u; i < 81u; i++) {
        depth_mgs[0][i] = move_group[i];
    }

    // loop until all possibilities are exausted
    var solution = false;
    while(depth_stack[0] < 9) {
        // try insert value
        let insert_index = last_index + depth_index + 1;
        var insert_value = depth_stack[depth_index] + 1;

        // skip if invalid move
        if ((depth_mgs[depth_index][insert_index] & (1u << (insert_value - 1))) == 0) {
            if ((depth_index > 0) && (depth_stack[depth_index] + 1 >= 9)) {
                // branch exausted, back out
                depth_stack[depth_index] = 0;
                depth_index--;
            }
            depth_stack[depth_index] += 1;
            continue;
        }

        let r = try_insert_cell(depth_mgs[depth_index], insert_index, insert_value);
        if (r.success) {
            // good insert
            if (last_index + depth_index + 2 >= 81) {
                // solution found
                move_group = r.mg;
                solution = true;
                break;
            }
            // else insert next depth node
            depth_index++;
            depth_stack[depth_index] = 0;
            // copy move group data
            depth_mgs[depth_index] = r.mg;
        } else {
            // bad insert
            if ((depth_index > 0) && (depth_stack[depth_index] + 1 >= 9)) {
                // branch exausted, back out
                depth_stack[depth_index] = 0;
                depth_index--;
            }
            // try next leaf
            depth_stack[depth_index] += 1;
        }
    }

    // if success
    if (solution) {
        // write output
        for (var i = 0u; i < 81u; i++) {
            // get value for each remaining item in move group
            var mg_value = 0u;
            for (var m = 0u; m < 9u; m++) {
                if((move_group[i] & (1u << m)) > 0) {
                    mg_value += m + 1;
                }
            }
            // write value
            storage_array[buffer_index].inner[i] = mg_value;
        }
    }
}

fn try_insert_cell(move_group: array<u32, 81>, index: u32, value: u32) -> TryInsertResult {
    // initial result
    var result = TryInsertResult(move_group, false);

    let r = index / 9; // insert row
    let c = index % 9; // insert col

    // for all cells
    for (var i = 0u; i < 81u; i++) {
        let ir = i / 9; // i row
        let ic = i % 9; // i col
        let bit = 1u << (value - 1);

        // if cell contains value
        if ((result.mg[i] & bit) == 0) {
            continue;
        }

        // if cell is target
        if (i == index) {
            // set only value as true
            result.mg[i] = bit;

        // if cell is same row
        } else if (r == ir) {
            // remove from move group
            result.mg[i] ^= bit;
            // check if out of moves
            if (result.mg[i] == 0) { return result; } // failed move
        // if cell is same col
        } else if (c == ic) {
            // remove from move group
            result.mg[i] ^= bit;
            // check if out of moves
            if (result.mg[i] == 0) { return result; } // failed move
        // if cell is in same 3x3 box
        } else if ((r / 3 == ir / 3) && (c / 3 == ic / 3)) {
            // remove from move group
            result.mg[i] ^= bit;
            // check if out of moves
            if (result.mg[i] == 0) { return result; } // failed move
        }
    }

    // no fails
    result.success = true;
    return result;
}
