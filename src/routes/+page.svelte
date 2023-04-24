<script lang="ts">
	import { default as wasm, find_shortest, animate_finding_shortest } from '../pkg/wasm.js';

	wasm(); // Promise

	let canvasWidth: number;
	let canvasHeight: number;
	let canvas: HTMLCanvasElement;

	let drone = { x: 0, y: 0 };
	let destinations: typeof drone[] = [];

	let shortest_path: number[] = [];

	let mode: 'drone' | 'destination' = 'drone';

	function canvasClick(e: PointerEvent) {
		let [x, y] = [Math.round(e.offsetX), Math.round(e.offsetY)];

		if (mode == 'drone') {
			// Left
			drone.x = x;
			drone.y = y;
			calc_shortest_length();
		} else if (mode == 'destination') {
			// Right
			destinations.push({ x, y });
		}

		drawCanvas();
	}

	let c: CanvasRenderingContext2D;
	let point_size = 6;
	function drawCanvas() {
		c = c ?? canvas.getContext('2d');
		if (!c) return;

		c.clearRect(0, 0, canvasWidth, canvasHeight);

		c.lineWidth = 1;

		if (shortest_path?.length > 1) {
			// Shortest path
			c.strokeStyle = 'black';
			c.beginPath();
			c.moveTo(drone.x, drone.y);
			for (let destInd of shortest_path.slice(1)) {
				let dest = destinations[destInd - 1];
				c.lineTo(dest.x, dest.y);
			}

			c.stroke();
		}

		c.lineWidth = 2;

		{
			// Destinations
			c.strokeStyle = 'red';
			for (let destination of destinations) {
				c.beginPath();
				c.arc(destination.x, destination.y, point_size, 0, Math.PI * 2);
				c.stroke();
			}
		}

		{
			// Drone
			c.strokeStyle = 'green';
			c.beginPath();
			c.arc(drone.x, drone.y, point_size, 0, Math.PI * 2);
			c.stroke();
		}
	}

	let shortestLength: string | null = null;

	function distance(p1: typeof drone, p2: typeof drone): number {
		return Math.sqrt(Math.pow(p2.x - p1.x, 2) + Math.pow(p2.y - p1.y, 2));
	}

	function calc_shortest_length() {
		if (!shortest_path) return;
		let length = 0;
		let last = drone;

		for (let i of shortest_path.slice(1)) {
			let dest = destinations[i - 1];
			length += distance(last, dest);
			last = dest;
		}

		shortestLength = `${length}px`;
	}

	function run(algorithm: string) {
		if (animated) {
			animate_finding_shortest(algorithm, [drone, ...destinations]).then((res) => {
				shortest_path = res;
				drawCanvas();
				calc_shortest_length();
			});
		} else {
			shortest_path = find_shortest(algorithm, [drone, ...destinations]);
			drawCanvas();
			calc_shortest_length();
		}
	}

	let animated = false;
</script>

<svelte:window
	on:animationframe={(e) => {
		shortest_path = e.detail;
		drawCanvas();
		calc_shortest_length();
	}}
	on:resize={() =>
		requestAnimationFrame(() => {
			requestAnimationFrame(drawCanvas);
		})}
/>

<div class="grid">
	<canvas
		bind:this={canvas}
		on:pointerdown={canvasClick}
		bind:clientHeight={canvasHeight}
		bind:clientWidth={canvasWidth}
		height={canvasHeight}
		width={canvasWidth}
		on:contextmenu|preventDefault
	/>
	<div class="stats">
		<label for="animated">
			Animated
			<input name="animated" type="checkbox" bind:checked={animated} />
		</label>
		<p>Current mode: {mode}</p>
		<p>Path length: {shortestLength ?? 'Not available'}</p>
	</div>
	<div class="algorithms">
		<button on:pointerdown={() => run('naive')}>Naive</button>
		<button on:pointerdown={() => run('closest')}>Closest</button>
	</div>
	<div class="modes">
		<button on:pointerdown={() => (mode = 'drone')}>Drone</button>
		<button on:pointerdown={() => (mode = 'destination')}>Destination</button>
	</div>
</div>

<style lang="postcss">
	.grid {
		display: grid;
		grid-template-rows: 25vh 50vh 25vh;
		grid-template-columns: 25vw 50vw 25vw;
		grid-template-areas:
			'. stats .'
			'. canvas modes'
			'. algorithms .';

		& > canvas {
			grid-area: canvas;
			border: 5px black solid;
			width: 100%;
			height: 100%;
		}

		& > .algorithms {
			grid-area: algorithms;
			display: flex;

			& > * {
				flex: 1;
			}
		}
		& > .modes {
			grid-area: modes;
			display: flex;
			flex-direction: column;

			& > * {
				flex: 1;
			}
		}

		& > .stats {
			grid-area: stats;
			display: flex;
			flex-direction: column;
			justify-content: end;
		}
	}
</style>
