<script lang="ts">
	import { getContext } from "svelte";

	import { debouncer } from "@graphite/utility-functions/debounce";
	import type { Editor } from "@graphite/wasm-communication/editor";
	import type { Widget, WidgetSpanColumn, WidgetSpanRow } from "@graphite/wasm-communication/messages";
	import { narrowWidgetProps, isWidgetSpanColumn, isWidgetSpanRow } from "@graphite/wasm-communication/messages";

	import BreadcrumbTrailButtons from "@graphite/components/widgets/buttons/BreadcrumbTrailButtons.svelte";
	import ColorButton from "@graphite/components/widgets/buttons/ColorButton.svelte";
	import IconButton from "@graphite/components/widgets/buttons/IconButton.svelte";
	import ParameterExposeButton from "@graphite/components/widgets/buttons/ParameterExposeButton.svelte";
	import PopoverButton from "@graphite/components/widgets/buttons/PopoverButton.svelte";
	import TextButton from "@graphite/components/widgets/buttons/TextButton.svelte";
	import WorkingColorsButton from "@graphite/components/widgets/buttons/WorkingColorsButton.svelte";
	import CheckboxInput from "@graphite/components/widgets/inputs/CheckboxInput.svelte";
	import CurveInput from "@graphite/components/widgets/inputs/CurveInput.svelte";
	import DropdownInput from "@graphite/components/widgets/inputs/DropdownInput.svelte";
	import FontInput from "@graphite/components/widgets/inputs/FontInput.svelte";
	import LayerReferenceInput from "@graphite/components/widgets/inputs/LayerReferenceInput.svelte";
	import NumberInput from "@graphite/components/widgets/inputs/NumberInput.svelte";
	import OptionalInput from "@graphite/components/widgets/inputs/OptionalInput.svelte";
	import PivotInput from "@graphite/components/widgets/inputs/PivotInput.svelte";
	import RadioInput from "@graphite/components/widgets/inputs/RadioInput.svelte";
	import TextAreaInput from "@graphite/components/widgets/inputs/TextAreaInput.svelte";
	import TextInput from "@graphite/components/widgets/inputs/TextInput.svelte";
	import IconLabel from "@graphite/components/widgets/labels/IconLabel.svelte";
	import ImageLabel from "@graphite/components/widgets/labels/ImageLabel.svelte";
	import Separator from "@graphite/components/widgets/labels/Separator.svelte";
	import TextLabel from "@graphite/components/widgets/labels/TextLabel.svelte";
	import WidgetLayout from "@graphite/components/widgets/WidgetLayout.svelte";

	const SUFFIX_WIDGETS = ["PopoverButton"];

	const editor = getContext<Editor>("editor");

	export let widgetData: WidgetSpanRow | WidgetSpanColumn;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	export let layoutTarget: any;

	$: direction = watchDirection(widgetData);
	$: widgets = watchWidgets(widgetData);
	$: widgetsAndNextSiblingIsSuffix = watchWidgetsAndNextSiblingIsSuffix(widgets);

	function watchDirection(widgetData: WidgetSpanRow | WidgetSpanColumn): "row" | "column" | undefined {
		if (isWidgetSpanRow(widgetData)) return "row";
		if (isWidgetSpanColumn(widgetData)) return "column";
	}

	function watchWidgets(widgetData: WidgetSpanRow | WidgetSpanColumn): Widget[] {
		let widgets: Widget[] = [];
		if (isWidgetSpanRow(widgetData)) widgets = widgetData.rowWidgets;
		else if (isWidgetSpanColumn(widgetData)) widgets = widgetData.columnWidgets;
		return widgets;
	}

	function watchWidgetsAndNextSiblingIsSuffix(widgets: Widget[]): [Widget, boolean][] {
		return widgets.map((widget, index): [Widget, boolean] => {
			// A suffix widget is one that joins up with this widget at the end with only a 1px gap.
			// It uses the CSS sibling selector to give its own left edge corners zero radius.
			// But this JS is needed to set its preceding sibling widget's right edge corners to zero radius.
			const nextSiblingIsSuffix = SUFFIX_WIDGETS.includes(widgets[index + 1]?.props.kind);

			return [widget, nextSiblingIsSuffix];
		});
	}

	function updateLayout(index: number, value: unknown) {
		editor.instance.updateLayout(layoutTarget, widgets[index].widgetId, value);
	}

	// TODO: This seems to work, but verify the correctness and terseness of this, it's adapted from https://stackoverflow.com/a/67434028/775283
	function exclude<T extends object>(props: T, additional?: (keyof T)[]): Omit<T, typeof additional extends Array<infer K> ? K : never> {
		const exclusions = ["kind", ...(additional || [])];

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		return Object.fromEntries(Object.entries(props).filter((entry) => !exclusions.includes(entry[0]))) as any;
	}
</script>

<!-- TODO: Refactor this component to use `<svelte:component this={attributesObject} />` to avoid all the separate conditional components -->
<!-- TODO: Also rename this component, and probably move the `widget-${direction}` wrapper to be part of `WidgetLayout.svelte` as part of its refactor -->

<div class="widget-span" class:row={direction === "row"} class:column={direction === "column"}>
	{#each widgetsAndNextSiblingIsSuffix as [component, nextIsSuffix], index}
		{@const checkboxInput = narrowWidgetProps(component.props, "CheckboxInput")}
		{#if checkboxInput}
			<CheckboxInput {...exclude(checkboxInput)} on:checked={({ detail }) => updateLayout(index, detail)} />
		{/if}
		{@const colorInput = narrowWidgetProps(component.props, "ColorButton")}
		{#if colorInput}
			<ColorButton {...exclude(colorInput)} on:value={({ detail }) => updateLayout(index, detail)} sharpRightCorners={nextIsSuffix} />
		{/if}
		{@const curvesInput = narrowWidgetProps(component.props, "CurveInput")}
		{#if curvesInput}
			<CurveInput {...exclude(curvesInput)} on:value={({ detail }) => debouncer((value) => updateLayout(index, value), { debounceTime: 120 }).debounceUpdateValue(detail)} />
		{/if}
		{@const dropdownInput = narrowWidgetProps(component.props, "DropdownInput")}
		{#if dropdownInput}
			<DropdownInput {...exclude(dropdownInput)} on:selectedIndex={({ detail }) => updateLayout(index, detail)} sharpRightCorners={nextIsSuffix} />
		{/if}
		{@const fontInput = narrowWidgetProps(component.props, "FontInput")}
		{#if fontInput}
			<FontInput {...exclude(fontInput)} on:changeFont={({ detail }) => updateLayout(index, detail)} sharpRightCorners={nextIsSuffix} />
		{/if}
		{@const parameterExposeButton = narrowWidgetProps(component.props, "ParameterExposeButton")}
		{#if parameterExposeButton}
			<ParameterExposeButton {...exclude(parameterExposeButton)} action={() => updateLayout(index, undefined)} />
		{/if}
		{@const iconButton = narrowWidgetProps(component.props, "IconButton")}
		{#if iconButton}
			<IconButton {...exclude(iconButton)} action={() => updateLayout(index, undefined)} sharpRightCorners={nextIsSuffix} />
		{/if}
		{@const iconLabel = narrowWidgetProps(component.props, "IconLabel")}
		{#if iconLabel}
			<IconLabel {...exclude(iconLabel)} />
		{/if}
		{@const imageLabel = narrowWidgetProps(component.props, "ImageLabel")}
		{#if imageLabel}
			<ImageLabel {...exclude(imageLabel)} />
		{/if}
		{@const layerReferenceInput = narrowWidgetProps(component.props, "LayerReferenceInput")}
		{#if layerReferenceInput}
			<LayerReferenceInput {...exclude(layerReferenceInput)} on:value={({ detail }) => updateLayout(index, detail)} />
		{/if}
		{@const numberInput = narrowWidgetProps(component.props, "NumberInput")}
		{#if numberInput}
			<NumberInput
				{...exclude(numberInput)}
				on:value={({ detail }) => debouncer((value) => updateLayout(index, value)).debounceUpdateValue(detail)}
				incrementCallbackIncrease={() => updateLayout(index, "Increment")}
				incrementCallbackDecrease={() => updateLayout(index, "Decrement")}
				sharpRightCorners={nextIsSuffix}
			/>
		{/if}
		{@const optionalInput = narrowWidgetProps(component.props, "OptionalInput")}
		{#if optionalInput}
			<OptionalInput {...exclude(optionalInput)} on:checked={({ detail }) => updateLayout(index, detail)} />
		{/if}
		{@const pivotInput = narrowWidgetProps(component.props, "PivotInput")}
		{#if pivotInput}
			<PivotInput {...exclude(pivotInput)} on:position={({ detail }) => updateLayout(index, detail)} />
		{/if}
		{@const popoverButton = narrowWidgetProps(component.props, "PopoverButton")}
		{#if popoverButton}
			<PopoverButton {...exclude(popoverButton, ["header", "text", "optionsWidget"])}>
				<TextLabel bold={true}>{popoverButton.header}</TextLabel>
				{#if popoverButton.optionsWidget?.length}
					<WidgetLayout layout={{ layout: popoverButton.optionsWidget, layoutTarget: layoutTarget }} />
				{:else}
					<TextLabel multiline={true}>{popoverButton.text}</TextLabel>
				{/if}
			</PopoverButton>
		{/if}
		{@const radioInput = narrowWidgetProps(component.props, "RadioInput")}
		{#if radioInput}
			<RadioInput {...exclude(radioInput)} on:selectedIndex={({ detail }) => updateLayout(index, detail)} sharpRightCorners={nextIsSuffix} />
		{/if}
		{@const separator = narrowWidgetProps(component.props, "Separator")}
		{#if separator}
			<Separator {...exclude(separator)} />
		{/if}
		{@const workingColorsButton = narrowWidgetProps(component.props, "WorkingColorsButton")}
		{#if workingColorsButton}
			<WorkingColorsButton {...exclude(workingColorsButton)} />
		{/if}
		{@const textAreaInput = narrowWidgetProps(component.props, "TextAreaInput")}
		{#if textAreaInput}
			<TextAreaInput {...exclude(textAreaInput)} on:commitText={({ detail }) => updateLayout(index, detail)} />
		{/if}
		{@const textButton = narrowWidgetProps(component.props, "TextButton")}
		{#if textButton}
			<TextButton {...exclude(textButton)} action={() => updateLayout(index, undefined)} sharpRightCorners={nextIsSuffix} />
		{/if}
		{@const breadcrumbTrailButtons = narrowWidgetProps(component.props, "BreadcrumbTrailButtons")}
		{#if breadcrumbTrailButtons}
			<BreadcrumbTrailButtons {...exclude(breadcrumbTrailButtons)} action={(index) => updateLayout(index, index)} />
		{/if}
		{@const textInput = narrowWidgetProps(component.props, "TextInput")}
		{#if textInput}
			<TextInput {...exclude(textInput)} on:commitText={({ detail }) => updateLayout(index, detail)} sharpRightCorners={nextIsSuffix} />
		{/if}
		{@const textLabel = narrowWidgetProps(component.props, "TextLabel")}
		{#if textLabel}
			<TextLabel {...exclude(textLabel, ["value"])}>{textLabel.value}</TextLabel>
		{/if}
	{/each}
</div>

<style lang="scss" global>
	.widget-span.column {
		flex: 0 0 auto;
		display: flex;
		flex-direction: column;
	}

	.widget-span.row {
		flex: 0 0 auto;
		display: flex;
		min-height: 32px;

		> * {
			--widget-height: 24px;
			margin: calc((24px - var(--widget-height)) / 2 + 4px) 0;
			min-height: var(--widget-height);

			&:not(.multiline) {
				line-height: var(--widget-height);
			}

			&.icon-label.size-12 {
				--widget-height: 12px;
			}

			&.icon-label.size-16 {
				--widget-height: 16px;
			}
		}

		// TODO: Target this in a better way than using the tooltip, which will break if changed, or when localized/translated
		.checkbox-input [title="Preserve Aspect Ratio"] {
			margin-bottom: -32px;
			position: relative;

			&::before,
			&::after {
				content: "";
				pointer-events: none;
				position: absolute;
				left: 8px;
				width: 1px;
				height: 16px;
				background: var(--color-7-middlegray);
			}

			&::before {
				top: calc(-4px - 16px);
			}

			&::after {
				bottom: calc(-4px - 16px);
			}
		}
	}
	// paddingpaddingpaddingpaddingpaddingpaddingpaddingpaddingpaddingpaddingpaddingpaddingpaddingpaddingpaddingpaddingpaddingpaddingpaddingpadding
</style>
