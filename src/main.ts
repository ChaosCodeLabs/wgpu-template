import './style.css';

async function setup(): Promise<HTMLCanvasElement> {
    // TODO: wasm setup
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
}

window.onload = async () => {
    const app = await setup();
    await render(app);
}
