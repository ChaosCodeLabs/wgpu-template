import './style.css';
import wasmInit, * as wasmCore from 'wasm-core';

let mouseX = 0;
let mouseY = 0;

async function setup(): Promise<wasmCore.App> {
    const canvas = <HTMLCanvasElement>(
        document.querySelector<HTMLCanvasElement>("#app")
    );
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;

    const app = wasmCore.App.setup(canvas);

    canvas.onmousemove = (event) => {
        mouseX = event.clientX - canvas.offsetLeft;
        mouseY = event.clientY - canvas.offsetTop;
    }
    return app;
}

function update(app: wasmCore.App, time: number, delta: number) {
    app.update(
        time,
        delta,
        { x: mouseX, y: mouseY },
    )
}

function render(app: wasmCore.App) {
    app.render()
}

window.onload = async () => {
    await wasmInit();
    const app = await setup();
    let prevTime = 0;

    function gameLoop(time: number) {
        const delta = (time - prevTime) / 1000.0;
        update(app, time / 1000.0, delta); // update time
        render(app);
        prevTime = time;
        requestAnimationFrame(gameLoop);
    }
    requestAnimationFrame(gameLoop);
}
