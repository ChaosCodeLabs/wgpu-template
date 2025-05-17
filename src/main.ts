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
    // TODO: render
    console.log("rendering... on", app);
    console.log(wasmCore.add(10, 20));
}

window.onload = async () => {
    const app = await setup();
    await render(app);
}
