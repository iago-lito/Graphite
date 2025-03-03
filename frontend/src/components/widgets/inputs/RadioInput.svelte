<script lang="ts">
	import { createEventDispatcher } from "svelte";

	import { type RadioEntries, type RadioEntryData } from "@graphite/wasm-communication/messages";

	import LayoutRow from "@graphite/components/layout/LayoutRow.svelte";
	import IconLabel from "@graphite/components/widgets/labels/IconLabel.svelte";
	import TextLabel from "@graphite/components/widgets/labels/TextLabel.svelte";

	// emits: ["update:selectedIndex"],
	const dispatch = createEventDispatcher<{ selectedIndex: number }>();

	export let entries: RadioEntries;
	export let selectedIndex: number | undefined = undefined;
	export let disabled = false;
	export let sharpRightCorners = false;

	function handleEntryClick(radioEntryData: RadioEntryData) {
		const index = entries.indexOf(radioEntryData);
		dispatch("selectedIndex", index);

		radioEntryData.action?.();
	}
</script>

<LayoutRow class="radio-input" classes={{ disabled }}>
	{#each entries as entry, index}
		<button
			class:active={index === selectedIndex}
			class:mixed={selectedIndex === undefined}
			class:disabled
			class:sharp-right-corners={index === entries.length - 1 && sharpRightCorners}
			on:click={() => handleEntryClick(entry)}
			title={entry.tooltip}
			tabindex={index === selectedIndex ? -1 : 0}
			{disabled}
		>
			{#if entry.icon}
				<IconLabel icon={entry.icon} />
			{/if}
			{#if entry.label}
				<TextLabel>{entry.label}</TextLabel>
			{/if}
		</button>
	{/each}
</LayoutRow>

<style lang="scss" global>
	.radio-input {
		button {
			background: var(--color-5-dullgray);
			fill: var(--color-e-nearwhite);
			height: 24px;
			margin: 0;
			padding: 0 4px;
			border: none;
			display: flex;
			align-items: center;
			justify-content: center;

			&.mixed {
				background: var(--color-4-dimgray);
			}

			&:hover {
				background: var(--color-6-lowergray);
				color: var(--color-f-white);

				svg {
					fill: var(--color-f-white);
				}
			}

			&.active {
				background: var(--color-e-nearwhite);
				color: var(--color-2-mildblack);

				svg {
					fill: var(--color-2-mildblack);
				}
			}

			&.disabled {
				background: var(--color-4-dimgray);
				color: var(--color-8-uppergray);

				svg {
					fill: var(--color-8-uppergray);
				}

				&.active {
					background: var(--color-8-uppergray);
					color: var(--color-2-mildblack);

					svg {
						fill: var(--color-2-mildblack);
					}
				}
			}

			& + button {
				margin-left: 1px;
			}

			&:first-of-type {
				border-radius: 2px 0 0 2px;
			}

			&:last-of-type {
				border-radius: 0 2px 2px 0;
			}
		}

		.text-label {
			margin: 0 4px;
			overflow: hidden;
		}

		&.combined-before button:first-of-type,
		&.combined-after button:last-of-type {
			border-radius: 0;
		}
	}
</style>
