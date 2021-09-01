// WASM and game state
var instance;
var gs_manager;

// Canvas and other stuff
var ctx;
var width;
var height;
var image_data;
var ASPECT_RATIO = 2;

const mem_size = 50;
const memory = new WebAssembly.Memory({initial: mem_size, shared: false});
var import_object = {
    env: {memory}
};
WebAssembly.instantiateStreaming(fetch('moo3d_wasm.wasm'), import_object)
.then(results => {
    if (test_wasm(results.instance)) {
        console.log('WASM PASSED');
        instance = results.instance;
        launch_init();
    }
    else {
        alert('WASM FAILED');
    }
});

function test_wasm(input_instance) {
    // Sanity check
    if (input_instance.exports.test_return_5() != 5) {
        return false;
    }

    // Return Uint8Array test
    let returned_arr = unwrapUint8Array(input_instance, input_instance.exports.test_return_arr());
    if (returned_arr.length != 5) {
        return false;
    }
    if (
        (returned_arr[0] != 1) || (returned_arr[1] != 2) || 
        (returned_arr[2] != 3) || (returned_arr[3] != 4) ||
        (returned_arr[4] != 5)
    ) {
        return false;
    }
    return true;
}

function launch_init() {
    let gameCanvas = document.getElementById('gameCanvas');

    width = Math.floor(window.innerWidth);
    height = Math.floor(Math.min(window.innerHeight, window.innerWidth / ASPECT_RATIO));

    gameCanvas.style.width = '' + width + 'px';
    gameCanvas.style.height = '' + height + 'px';
    
    width = Math.floor(width * 0.66);
    height = Math.floor(height * 0.66);

    gameCanvas.height = height;
    gameCanvas.width = width;

    ctx = gameCanvas.getContext('2d');

    gs_manager = instance.exports.make_game_state(width, height);

    let raw_data = instance.exports.get_pixel_data(gs_manager)

    image_data = new ImageData(unwrapUint8ClampedArray(instance, raw_data), width);

    requestAnimationFrame(renderLoop);
}

let translate_state = [0, 0, 0];
let translate_look_state = [0, 0, 0];

function renderLoop(curr_time) {
    instance.exports.translate_camera(gs_manager, translate_state[0], translate_state[1], translate_state[2]);
    instance.exports.translate_camera_look(gs_manager, translate_look_state[0], translate_look_state[1], translate_look_state[2]);

    requestAnimationFrame(renderLoop);
    instance.exports.render_game(gs_manager, curr_time);
    /*
    let raw_data = instance.exports.get_pixel_data(gs_manager)

    ctx.putImageData(new ImageData(unwrapUint8ClampedArray(instance, raw_data), width), 0, 0);

    instance.exports.free_uint8_arr(raw_data);
    */
    ctx.putImageData(image_data, 0, 0);
}

document.addEventListener('keydown', function(e) {
    if (e.key == 'w') {
        translate_state = [0, 50, 0];   
    }
    else if (e.key == 'a') {
        translate_state = [-50, 0, 0];   
    }
    else if (e.key == 'd') {
        translate_state = [50, 0, 0];   
    }
    else if (e.key == 's') {
        translate_state = [0, -50, 0];   
    }
    else if (e.key == 'q') {
        translate_state = [0, 0, 50];   
    }
    else if (e.key == 'e') {
        translate_state = [0, 0, -50];   
    }
    else if (e.key == 'ArrowUp') {
        translate_look_state = [0, 0.1, 0];
    }
    else if (e.key == 'ArrowLeft') {
        translate_look_state = [-0.1, 0, 0];
    }
    else if (e.key == 'ArrowDown') {
        translate_look_state = [0, -0.1, 0];
    }
    else if (e.key == 'ArrowRight') {
        translate_look_state = [0.1, 0, 0];
    }
});

document.addEventListener('keyup', function(e) {
    switch (e.key) {
        case 'w':
        case 'a':
        case 's':
        case 'd':
        case 'q':
        case 'e':
            translate_state = [0, 0, 0];
        break;
        case 'ArrowUp':
        case 'ArrowDown':
        case 'ArrowLeft':
        case 'ArrowRight':
            translate_look_state = [0, 0, 0];
        break;
    }
})