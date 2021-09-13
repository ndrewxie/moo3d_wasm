let base16_table = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];

function reverse_base16_lookup(input) {
    for (let k = 0; k < base16_table.length; k++) {
        if (base16_table[k] == input) {
            return k;
        }
    }
}

function uint8_array_from_base16(input) {
    let to_return = [];
    for (let j = 0; j < input.length; j += 2) {
        let a = reverse_base16_lookup(input[j]);
        let b = reverse_base16_lookup(input[j+1]);
        to_return.push(a * 16 + b);        
    }
    return to_return;
}