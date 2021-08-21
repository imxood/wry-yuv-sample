<script>
	import YUVCanvas from "yuv-canvas";
	import YUVBuffer from "yuv-buffer";
	import Module from "../api/ffmpeg_264_265";
	import { onMount } from "svelte";

	let socket;
	let yuv_canvas;

	let canvas_id = "playCanvas";
	let yellow = false;

	onMount(async function () {
		// 加载 ffmpeg wasm
		Module()
			.then((ffmpeg) => {
				socket = new WebSocket("ws://" + location.host + "/ws");

				socket.onclose = (e) => {
					if (yuv_canvas) {
						yuv_canvas.clear();
					}
				};

				socket.onerror = (e) => {
					console.log("Socket open failed, error: " + e);
				};
				socket.onopen = () => {
					yuv_canvas = YUVCanvas.attach(
						document.getElementById(canvas_id)
					);
					// open decoder
					open_decoder(ffmpeg, yuv_canvas);
				};
				socket.onmessage = (event) => {
					if (event.data) {
						if (event.data.arrayBuffer) {
							event.data.arrayBuffer().then(async (buffer) => {
								await decode(ffmpeg, buffer);
							});
						}
					}
				};
			})
			.catch(function (e) {
				console.log("load ffmpeg wasm error: " + e);
			});

		console.log("App mounted");
	});

	async function decode(ffmpeg, buffer) {
		let typedArray = new Uint8Array(buffer);
		let size = typedArray.length;
		let cacheBuffer = ffmpeg._malloc(size);
		ffmpeg.HEAPU8.set(typedArray, cacheBuffer);
		let ret = await ffmpeg._decodeData(cacheBuffer, size, 0);
		if (cacheBuffer != null) {
			ffmpeg._free(cacheBuffer);
			cacheBuffer = null;
		}
	}

	function open_decoder(ffmpeg, yuv_canvas) {
		let videoSize = 0;
		let LOG_LEVEL_WASM = 1;
		let IS_H265 = 0;
		let videoCallback = ffmpeg.addFunction(function (
			addr_y,
			addr_u,
			addr_v,
			stride_y,
			stride_u,
			stride_v,
			width,
			height,
			pts
		) {
			console.log(
				"[%d]In video callback, size = %d * %d",
				++videoSize,
				width,
				height
			);
			let y_width = width;
			let y_height = height;

			let uv_width = width / 2;
			let uv_height = height / 2;

			let y_data = new Uint8Array(stride_y * y_height);
			for (var pos = 0, i = 0; i < y_height; i++) {
				let src = addr_y + i * stride_y;
				let tmp = ffmpeg.HEAPU8.subarray(src, src + stride_y);
				tmp = new Uint8Array(tmp);
				y_data.set(tmp, pos);
				pos += tmp.length;
			}

			let u_data = new Uint8Array(stride_u * uv_height);
			for (var pos = 0, i = 0; i < uv_height; i++) {
				let src = addr_u + i * stride_u;
				let tmp = ffmpeg.HEAPU8.subarray(src, src + stride_u);
				tmp = new Uint8Array(tmp);
				u_data.set(tmp, pos);
				pos += tmp.length;
			}

			let v_data = new Uint8Array(stride_v * uv_height);
			for (var pos = 0, i = 0; i < uv_height; i++) {
				let src = addr_v + i * stride_v;
				let tmp = ffmpeg.HEAPU8.subarray(src, src + stride_v);
				tmp = new Uint8Array(tmp);
				v_data.set(tmp, pos);
				pos += tmp.length;
			}

			let format = YUVBuffer.format({
				width,
				height,
				chromaWidth: width / 2,
				chromaHeight: height / 2,
			});
			let y = YUVBuffer.allocPlane(y_width, y_height, y_data, y_width);
			let u = YUVBuffer.allocPlane(uv_width, uv_height, u_data, uv_width);
			let v = YUVBuffer.allocPlane(uv_width, uv_height, v_data, uv_width);
			let frame = YUVBuffer.frame(format, y, u, v);
			// webgl render the yuv420 data
			yuv_canvas.drawFrame(frame);
		},
		"viiiiiiiii");

		let ret = ffmpeg._openDecoder(IS_H265, videoCallback, LOG_LEVEL_WASM);
		if (ret == 0) {
			console.log("openDecoder success");
		} else {
			console.error("openDecoder failed with error", ret);
			return;
		}
	}

	function test_area_clicked() {
		yellow = !yellow;
		console.log("clicked", yellow);
	}
</script>

<div style="height:100%;">
	<canvas id="playCanvas" style="width:100%; height: 100%;" />
	<div id="test_area1">test CSS hover</div>
	<div id="test_area2" class:yellow on:click={test_area_clicked}>
		test js click
	</div>
</div>

<style>
	#test_area1 {
		width: 200px;
		height: 40px;
		position: absolute;
		top: 20px;
		right: 20px;
		background-color: white;
		font-size: 30px;
	}
	#test_area1:hover {
		background-color: yellow;
	}

	#test_area2 {
		width: 200px;
		height: 40px;
		position: absolute;
		top: 100px;
		right: 20px;
		background-color: white;
		font-size: 30px;
	}
	.yellow {
		background-color: yellow !important;
	}
</style>
