function unwrapUint8Array(instance, pointer) {
    return new Uint8Array(instance.exports.memory.buffer, instance.exports.get_array_data(pointer), instance.exports.get_array_length(pointer));
}

function unwrapUint8ClampedArray(instance, pointer) {
    return new Uint8ClampedArray(instance.exports.memory.buffer, instance.exports.get_array_data(pointer), instance.exports.get_array_length(pointer));
}