use super::tool_prelude::*;
use crate::messages::portfolio::document::node_graph::VectorDataModification;
use crate::messages::tool::common_functionality::color_selector::{ToolColorOptions, ToolColorType};
use crate::messages::tool::common_functionality::graph_modification_utils;

use document_legacy::document_metadata::LayerNodeIdentifier;
use graphene_core::uuid::generate_uuid;
use graphene_core::vector::style::{Fill, Stroke};
use graphene_core::Color;

use bezier_rs::ManipulatorGroup;
use glam::DVec2;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct FreehandTool {
	fsm_state: FreehandToolFsmState,
	data: FreehandToolData,
	options: FreehandOptions,
}

pub struct FreehandOptions {
	line_weight: f64,
	fill: ToolColorOptions,
	stroke: ToolColorOptions,
}

impl Default for FreehandOptions {
	fn default() -> Self {
		Self {
			line_weight: 5.,
			fill: ToolColorOptions::new_none(),
			stroke: ToolColorOptions::new_primary(),
		}
	}
}

#[remain::sorted]
#[impl_message(Message, ToolMessage, Freehand)]
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, specta::Type)]
pub enum FreehandToolMessage {
	// Standard messages
	#[remain::unsorted]
	Abort,
	#[remain::unsorted]
	WorkingColorChanged,

	// Tool-specific messages
	DragStart,
	DragStop,
	PointerMove,
	UpdateOptions(FreehandOptionsUpdate),
}

#[remain::sorted]
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, specta::Type)]
pub enum FreehandOptionsUpdate {
	FillColor(Option<Color>),
	FillColorType(ToolColorType),
	LineWeight(f64),
	StrokeColor(Option<Color>),
	StrokeColorType(ToolColorType),
	WorkingColors(Option<Color>, Option<Color>),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum FreehandToolFsmState {
	#[default]
	Ready,
	Drawing,
}

impl ToolMetadata for FreehandTool {
	fn icon_name(&self) -> String {
		"VectorFreehandTool".into()
	}
	fn tooltip(&self) -> String {
		"Freehand Tool".into()
	}
	fn tool_type(&self) -> crate::messages::tool::utility_types::ToolType {
		ToolType::Freehand
	}
}

fn create_weight_widget(line_weight: f64) -> WidgetHolder {
	NumberInput::new(Some(line_weight))
		.unit(" px")
		.label("Weight")
		.min(1.)
		.max((1u64 << std::f64::MANTISSA_DIGITS) as f64)
		.on_update(|number_input: &NumberInput| FreehandToolMessage::UpdateOptions(FreehandOptionsUpdate::LineWeight(number_input.value.unwrap())).into())
		.widget_holder()
}

impl LayoutHolder for FreehandTool {
	fn layout(&self) -> Layout {
		let mut widgets = self.options.fill.create_widgets(
			"Fill",
			true,
			|_| FreehandToolMessage::UpdateOptions(FreehandOptionsUpdate::FillColor(None)).into(),
			|color_type: ToolColorType| WidgetCallback::new(move |_| FreehandToolMessage::UpdateOptions(FreehandOptionsUpdate::FillColorType(color_type.clone())).into()),
			|color: &ColorButton| FreehandToolMessage::UpdateOptions(FreehandOptionsUpdate::FillColor(color.value)).into(),
		);

		widgets.push(Separator::new(SeparatorType::Section).widget_holder());

		widgets.append(&mut self.options.stroke.create_widgets(
			"Stroke",
			true,
			|_| FreehandToolMessage::UpdateOptions(FreehandOptionsUpdate::StrokeColor(None)).into(),
			|color_type: ToolColorType| WidgetCallback::new(move |_| FreehandToolMessage::UpdateOptions(FreehandOptionsUpdate::StrokeColorType(color_type.clone())).into()),
			|color: &ColorButton| FreehandToolMessage::UpdateOptions(FreehandOptionsUpdate::StrokeColor(color.value)).into(),
		));
		widgets.push(Separator::new(SeparatorType::Unrelated).widget_holder());
		widgets.push(create_weight_widget(self.options.line_weight));

		Layout::WidgetLayout(WidgetLayout::new(vec![LayoutGroup::Row { widgets }]))
	}
}

impl<'a> MessageHandler<ToolMessage, &mut ToolActionHandlerData<'a>> for FreehandTool {
	fn process_message(&mut self, message: ToolMessage, responses: &mut VecDeque<Message>, tool_data: &mut ToolActionHandlerData<'a>) {
		let ToolMessage::Freehand(FreehandToolMessage::UpdateOptions(action)) = message else {
			self.fsm_state.process_event(message, &mut self.data, tool_data, &self.options, responses, true);
			return;
		};
		match action {
			FreehandOptionsUpdate::FillColor(color) => {
				self.options.fill.custom_color = color;
				self.options.fill.color_type = ToolColorType::Custom;
			}
			FreehandOptionsUpdate::FillColorType(color_type) => self.options.fill.color_type = color_type,
			FreehandOptionsUpdate::LineWeight(line_weight) => self.options.line_weight = line_weight,
			FreehandOptionsUpdate::StrokeColor(color) => {
				self.options.stroke.custom_color = color;
				self.options.stroke.color_type = ToolColorType::Custom;
			}
			FreehandOptionsUpdate::StrokeColorType(color_type) => self.options.stroke.color_type = color_type,
			FreehandOptionsUpdate::WorkingColors(primary, secondary) => {
				self.options.stroke.primary_working_color = primary;
				self.options.stroke.secondary_working_color = secondary;
				self.options.fill.primary_working_color = primary;
				self.options.fill.secondary_working_color = secondary;
			}
		}

		self.send_layout(responses, LayoutTarget::ToolOptions);
	}

	fn actions(&self) -> ActionList {
		use FreehandToolFsmState::*;

		match self.fsm_state {
			Ready => actions!(FreehandToolMessageDiscriminant;
				DragStart,
				DragStop,
				Abort,
			),
			Drawing => actions!(FreehandToolMessageDiscriminant;
				DragStop,
				PointerMove,
				Abort,
			),
		}
	}
}

impl ToolTransition for FreehandTool {
	fn event_to_message_map(&self) -> EventToMessageMap {
		EventToMessageMap {
			tool_abort: Some(FreehandToolMessage::Abort.into()),
			working_color_changed: Some(FreehandToolMessage::WorkingColorChanged.into()),
			..Default::default()
		}
	}
}

#[derive(Clone, Debug, Default)]
struct FreehandToolData {
	last_point: DVec2,
	dragged: bool,
	weight: f64,
	layer: Option<LayerNodeIdentifier>,
}

impl Fsm for FreehandToolFsmState {
	type ToolData = FreehandToolData;
	type ToolOptions = FreehandOptions;

	fn transition(self, event: ToolMessage, tool_data: &mut Self::ToolData, tool_action_data: &mut ToolActionHandlerData, tool_options: &Self::ToolOptions, responses: &mut VecDeque<Message>) -> Self {
		let ToolActionHandlerData {
			document, global_tool_data, input, ..
		} = tool_action_data;

		let transform = document.metadata().document_to_viewport;

		let ToolMessage::Freehand(event) = event else {
			return self;
		};
		match (self, event) {
			(FreehandToolFsmState::Ready, FreehandToolMessage::DragStart) => {
				responses.add(DocumentMessage::StartTransaction);
				responses.add(DocumentMessage::DeselectAllLayers);

				let pos = transform.inverse().transform_point2(input.mouse.position);

				tool_data.dragged = false;
				tool_data.last_point = pos;

				tool_data.weight = tool_options.line_weight;

				let subpath = bezier_rs::Subpath::from_anchors([pos], false);

				let layer = graph_modification_utils::new_vector_layer(vec![subpath], generate_uuid(), document.new_layer_parent(), responses);
				tool_data.layer = Some(layer);

				responses.add(GraphOperationMessage::FillSet {
					layer: layer.to_path(),
					fill: if let Some(color) = tool_options.fill.active_color() { Fill::Solid(color) } else { Fill::None },
				});

				responses.add(GraphOperationMessage::StrokeSet {
					layer: layer.to_path(),
					stroke: Stroke::new(tool_options.stroke.active_color(), tool_data.weight),
				});

				FreehandToolFsmState::Drawing
			}
			(FreehandToolFsmState::Drawing, FreehandToolMessage::PointerMove) => {
				let pos = transform.inverse().transform_point2(input.mouse.position);

				if tool_data.last_point != pos {
					if let Some(layer) = tool_data.layer.clone() {
						let manipulator_group = ManipulatorGroup::new_anchor(pos);
						let modification = VectorDataModification::AddEndManipulatorGroup { subpath_index: 0, manipulator_group };
						responses.add(GraphOperationMessage::Vector { layer: layer.to_path(), modification });
						tool_data.dragged = true;
						tool_data.last_point = pos;
					}
				}

				FreehandToolFsmState::Drawing
			}
			(FreehandToolFsmState::Drawing, FreehandToolMessage::DragStop | FreehandToolMessage::Abort) => {
				if tool_data.dragged {
					responses.add(DocumentMessage::CommitTransaction);
				} else {
					responses.add(DocumentMessage::AbortTransaction);
				}

				tool_data.layer = None;

				FreehandToolFsmState::Ready
			}
			(_, FreehandToolMessage::WorkingColorChanged) => {
				responses.add(FreehandToolMessage::UpdateOptions(FreehandOptionsUpdate::WorkingColors(
					Some(global_tool_data.primary_color),
					Some(global_tool_data.secondary_color),
				)));
				self
			}
			_ => self,
		}
	}

	fn update_hints(&self, responses: &mut VecDeque<Message>) {
		let hint_data = match self {
			FreehandToolFsmState::Ready => HintData(vec![HintGroup(vec![HintInfo::mouse(MouseMotion::LmbDrag, "Draw Polyline")])]),
			FreehandToolFsmState::Drawing => HintData(vec![]),
		};

		responses.add(FrontendMessage::UpdateInputHints { hint_data });
	}

	fn update_cursor(&self, responses: &mut VecDeque<Message>) {
		responses.add(FrontendMessage::UpdateMouseCursor { cursor: MouseCursorIcon::Default });
	}
}
