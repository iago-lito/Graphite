/* eslint-disable max-classes-per-file */

import { writable } from "svelte/store";

import { copyToClipboardFileURL } from "@graphite/io-managers/clipboard";
import { downloadFileText, downloadFileBlob, upload } from "@graphite/utility-functions/files";
import { extractPixelData, imageToPNG, rasterizeSVG, rasterizeSVGCanvas } from "@graphite/utility-functions/rasterization";
import { type Editor } from "@graphite/wasm-communication/editor";
import {
	type FrontendDocumentDetails,
	TriggerCopyToClipboardBlobUrl,
	TriggerFetchAndOpenDocument,
	TriggerDownloadBlobUrl,
	TriggerDownloadRaster,
	TriggerDownloadTextFile,
	TriggerImport,
	TriggerOpenDocument,
	TriggerRasterizeRegionBelowLayer,
	TriggerRevokeBlobUrl,
	UpdateActiveDocument,
	UpdateImageData,
	UpdateOpenDocumentsList,
} from "@graphite/wasm-communication/messages";

// eslint-disable-next-line @typescript-eslint/explicit-function-return-type
export function createPortfolioState(editor: Editor) {
	const { subscribe, update } = writable({
		unsaved: false,
		documents: [] as FrontendDocumentDetails[],
		activeDocumentIndex: 0,
	});

	// Set up message subscriptions on creation
	editor.subscriptions.subscribeJsMessage(UpdateOpenDocumentsList, (updateOpenDocumentList) => {
		update((state) => {
			state.documents = updateOpenDocumentList.openDocuments;
			return state;
		});
	});
	editor.subscriptions.subscribeJsMessage(UpdateActiveDocument, (updateActiveDocument) => {
		update((state) => {
			// Assume we receive a correct document id
			const activeId = state.documents.findIndex((doc) => doc.id === updateActiveDocument.documentId);
			state.activeDocumentIndex = activeId;
			return state;
		});
	});
	editor.subscriptions.subscribeJsMessage(TriggerFetchAndOpenDocument, async (triggerFetchAndOpenDocument) => {
		try {
			const url = new URL(triggerFetchAndOpenDocument.url);
			const data = await fetch(url);

			const filename = url.pathname.split("/").pop() || "Untitled";
			const content = await data.text();

			editor.instance.openDocumentFile(filename, content);
		} catch {
			editor.instance.errorDialog("Failed to open document", "The file could not be reached over the internet. You may be offline, or it may be missing.");
		}
	});
	editor.subscriptions.subscribeJsMessage(TriggerOpenDocument, async () => {
		const extension = editor.instance.fileSaveSuffix();
		const data = await upload(extension, "text");
		editor.instance.openDocumentFile(data.filename, data.content);
	});
	editor.subscriptions.subscribeJsMessage(TriggerImport, async () => {
		const data = await upload("image/*", "data");
		const imageData = await extractPixelData(new Blob([data.content], { type: data.type }));
		editor.instance.pasteImage(new Uint8Array(imageData.data), imageData.width, imageData.height);
	});
	editor.subscriptions.subscribeJsMessage(TriggerDownloadTextFile, (triggerFileDownload) => {
		downloadFileText(triggerFileDownload.name, triggerFileDownload.document);
	});
	editor.subscriptions.subscribeJsMessage(TriggerDownloadBlobUrl, async (triggerDownloadBlobUrl) => {
		const data = await fetch(triggerDownloadBlobUrl.blobUrl);
		const blob = await data.blob();

		// TODO: Remove this if/when we end up returning PNG directly from the backend
		const pngBlob = await imageToPNG(blob);

		downloadFileBlob(triggerDownloadBlobUrl.layerName, pngBlob);
	});
	editor.subscriptions.subscribeJsMessage(TriggerCopyToClipboardBlobUrl, (triggerDownloadBlobUrl) => {
		copyToClipboardFileURL(triggerDownloadBlobUrl.blobUrl);
	});
	editor.subscriptions.subscribeJsMessage(TriggerDownloadRaster, async (triggerRasterDownload) => {
		const { svg, name, mime, size } = triggerRasterDownload;

		// Fill the canvas with white if it'll be a JPEG (which does not support transparency and defaults to black)
		const backgroundColor = mime.endsWith("jpeg") ? "white" : undefined;

		// Rasterize the SVG to an image file
		try {
			const blob = await rasterizeSVG(svg, size.x, size.y, mime, backgroundColor);

			// Have the browser download the file to the user's disk
			downloadFileBlob(name, blob);
		} catch {
			// Fail silently if there's an error rasterizing the SVG, such as a zero-sized image
		}
	});
	editor.subscriptions.subscribeJsMessage(UpdateImageData, (updateImageData) => {
		updateImageData.imageData.forEach(async (element) => {
			const buffer = new Uint8Array(element.imageData.values()).buffer;
			const blob = new Blob([buffer], { type: element.mime });

			const blobURL = URL.createObjectURL(blob);

			// Pre-decode the image so it is ready to be drawn instantly once it's placed into the viewport SVG
			const image = new Image();
			image.src = blobURL;
			await image.decode();

			editor.instance.setImageBlobURL(updateImageData.documentId, element.path, element.nodeId, blobURL, image.naturalWidth, image.naturalHeight, element.transform);
		});
	});
	editor.subscriptions.subscribeJsMessage(TriggerRasterizeRegionBelowLayer, async (triggerRasterizeRegionBelowLayer) => {
		const { documentId, layerPath, svg, size } = triggerRasterizeRegionBelowLayer;

		// Rasterize the SVG to an image file
		try {
			if (size[0] >= 1 && size[1] >= 1) {
				const imageData = (await rasterizeSVGCanvas(svg, size[0], size[1])).getContext("2d")?.getImageData(0, 0, size[0], size[1]);
				if (!imageData) return;

				editor.instance.renderGraphUsingRasterizedRegionBelowLayer(documentId, layerPath, new Uint8Array(imageData.data), imageData.width, imageData.height);
			}
		} catch (e) {
			// getImageData may throw an exception if the resolution is too high
			// eslint-disable-next-line no-console
			console.error("Failed to rasterize the SVG canvas in JS to be sent back to Rust:", e);
		}
	});
	editor.subscriptions.subscribeJsMessage(TriggerRevokeBlobUrl, async (triggerRevokeBlobUrl) => {
		URL.revokeObjectURL(triggerRevokeBlobUrl.url);
	});

	return {
		subscribe,
	};
}
export type PortfolioState = ReturnType<typeof createPortfolioState>;
