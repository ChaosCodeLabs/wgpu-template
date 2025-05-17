import './style.css';
import wasmInit, * as wasmCore from 'wasm-core';

async function setup(): Promise<HTMLCanvasElement> {
    await wasmInit();
    const app = <HTMLCanvasElement>(
        document.querySelector<HTMLCanvasElement>("#app")
    );
    app.width = app.clientWidth;
    app.height = app.clientHeight;
    return app;
}

async function render(app: HTMLCanvasElement) {
    await wasmCore.run(app);
}

window.onload = async () => {
    const app = await setup();
    await render(app);
}
