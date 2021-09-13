// WASM and game state
var instance;
var gs_manager;

// Canvas and other stuff
var ctx;
var width;
var height;
var image_data;
var ASPECT_RATIO = 2;

// Control variables
let translate_state = [0, 0, 0];
let translate_look_state = [0, 0];
let deltas = [0, 0, 0];

let last_frame = 0;
let fps = 0;

const mem_size = 50;
const memory = new WebAssembly.Memory({initial: mem_size, shared: false});
var import_object = {
    env: {memory}
};

let wasm_request = WebAssembly.instantiateStreaming(fetch('m3d_wasm.wasm'), import_object);
let image_request = fetch('images.txt');
Promise.all([wasm_request, image_request]).then(async (promised_values) => {
    let wasm_instance = promised_values[0];
    let fetched_images = promised_values[1];

    if (test_wasm(wasm_instance.instance)) {
        console.log('WASM PASSED');
        instance = wasm_instance.instance;
        await launch_init(fetched_images);
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

async function launch_init(img_response) {
    let loaded_images = await (async function() {
        let tentative_images = [];

        let loaded_text = (await img_response.text()).split('\n');
        for (let j = 0; j < loaded_text.length; j++) {
            let to_parse = uint8_array_from_base16(loaded_text[j]);
            let colors = (function() {
                let colors_acc = [];

                let num_colors_a = to_parse.shift();
                let num_colors_b = to_parse.shift();
                let num_colors = num_colors_a * 256 + num_colors_b;
                for (let k = 0; k < num_colors; k++) {
                    let r = to_parse.shift();
                    let g = to_parse.shift();
                    let b = to_parse.shift();
                    let a = to_parse.shift();
                    colors_acc.push([r, g, b, a]);
                }
                return colors_acc;
            })();
            while (to_parse.length > 0) {
                let num_repeats = (function() {
                    let num_repeats_a = to_parse.shift();
                    if (num_repeats_a >= 255) {
                        let num_repeats_c = to_parse.shift();
                        let num_repeats_d = to_parse.shift();
                        return num_repeats_c * 256 + num_repeats_d;
                    }
                    else {
                        return num_repeats_a;
                    }
                })();
                let run_color = to_parse.shift();

                for (let k = 0; k < num_repeats; k++) {
                    tentative_images.push(...colors[run_color]);
                }
            }
        }
        console.log(tentative_images);
        if (tentative_images.length % 4 == 0) {
            console.log("IMAGE LOAD PASSED");
        }
        else {
            alert("IMAGE LOAD FAILED");
        }
        return new Uint8Array(tentative_images);
    })();
    let boxed_images = uint8ToWasm(instance, loaded_images);

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

    gs_manager = instance.exports.make_game_state(width, height, boxed_images);
    instance.exports.free_uint8_arr(boxed_images);

    let raw_data = instance.exports.get_pixel_data(gs_manager)

    image_data = new ImageData(unwrapUint8ClampedArray(instance, raw_data), width);

    requestAnimationFrame(renderLoop);
}

function renderLoop(curr_time) {
    fps = 1000 / (curr_time - last_frame);
    last_frame = curr_time;

    deltas[0] += translate_state[0];
    deltas[1] += translate_state[1];
    deltas[2] += translate_state[2];
    
    instance.exports.translate_camera(gs_manager, translate_state[0], translate_state[1], translate_state[2]);
    instance.exports.rotate_camera(gs_manager, translate_look_state[0], translate_look_state[1]);

    requestAnimationFrame(renderLoop);
    instance.exports.render_game(gs_manager, curr_time);

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
        translate_look_state = [0, 0.1];
    }
    else if (e.key == 'ArrowLeft') {
        translate_look_state = [-0.1, 0];
    }
    else if (e.key == 'ArrowDown') {
        translate_look_state = [0, -0.1];
    }
    else if (e.key == 'ArrowRight') {
        translate_look_state = [0.1, 0];
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
            translate_look_state = [0, 0];
        break;
    }
});

let fps_meter = document.getElementById('fpsMeter');
setInterval(function() {
    fps_meter.innerText = 
        'FPS: ' + (Math.round(fps * 100) / 100) + '\n' +
        'Deltas: [' + deltas[0] + ', ' + deltas[1] + ', ' + deltas[2] + ']\n' +
        'Width: ' + width + ", height: " + height;
}, 2000);