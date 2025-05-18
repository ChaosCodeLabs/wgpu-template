import './style.css';
import wasmInit, * as wasmCore from 'wasm-core';

async function setup(): Promise<wasmCore.App> {
    await wasmInit();
    const canvas = <HTMLCanvasElement>(
        document.querySelector<HTMLCanvasElement>("#app")
    );
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;
    const app = wasmCore.App.setup(canvas);
    return app;
}

function render(app: wasmCore.App) {
    app.render()
}

window.onload = async () => {
    const app = await setup();
    render(app);
}
