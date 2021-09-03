function unwrapUint8Array(instance, pointer) {
    return new Uint8Array(instance.exports.memory.buffer, instance.exports.get_array_data(pointer), instance.exports.get_array_length(pointer));
}

function unwrapUint8ClampedArray(instance, pointer) {
    return new Uint8ClampedArray(instance.exports.memory.buffer, instance.exports.get_array_data(pointer), instance.exports.get_array_length(pointer));
}

function uint8ToWasm(instance, to_pass) {
    let wasm_array = instance.exports.new_uint8_arr(to_pass.length);
    let to_fill = new Uint8Array(instance.exports.memory.buffer, instance.exports.get_array_data(wasm_array), to_pass.length);
    to_fill.set(to_pass);
    return wasm_array;
}