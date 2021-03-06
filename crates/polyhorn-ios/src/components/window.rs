use polyhorn_channel::{use_channel, Sender};
use polyhorn_core::CommandBuffer;
use polyhorn_ios_sys::polykit::PLYWindow;
use polyhorn_ui::geometry::{Dimension, Size};
use polyhorn_ui::hooks::SafeAreaInsets;
use polyhorn_ui::styles::{FlexDirection, Position, Relative, ViewStyle};
use std::rc::Rc;

use crate::prelude::*;
use crate::raw::{Builtin, Container, OpaqueContainer};
use crate::Key;

impl Container for PLYWindow {
    fn mount(&mut self, child: &mut OpaqueContainer) {
        if let Some(view) = child.container().to_view() {
            self.root_view_controller().view_mut().add_subview(&view)
        }
    }

    fn unmount(&mut self) {
        // A `UIWindow` is automatically closed when its retain count drops to
        // zero.
    }

    fn to_window(&self) -> Option<PLYWindow> {
        Some(self.clone())
    }
}

impl Component for Window {
    fn render(&self, manager: &mut Manager) -> Element {
        let reference = use_reference!(manager);
        let reference_clone = reference.clone();

        let insets = use_reference!(manager);
        let marker = use_state!(manager, ());

        let mut channel: Sender<SafeAreaInsets> = use_channel!(manager, {
            let insets = insets.clone();

            move |mut receiver| async move {
                while let Some(message) = receiver.next().await {
                    if insets.to_owned() == Some(message) {
                        continue;
                    }

                    insets.replace(message);
                    marker.replace(());
                }
            }
        });

        use_effect!(manager, move |buffer| {
            let id = match reference_clone.as_copy() {
                Some(id) => id,
                None => return,
            };

            buffer.mutate(&[id], move |containers| {
                let container = &mut containers[0];

                let frame = match container.downcast_mut::<PLYWindow>() {
                    Some(window) => {
                        let mut view_controller = window.root_view_controller();
                        let view = view_controller.view_mut();
                        view.set_needs_layout();
                        view.frame()
                    }
                    None => return,
                };

                let mut layout = match container.layout() {
                    Some(layout) => layout.clone(),
                    None => return,
                };

                let insets = container
                    .container()
                    .to_window()
                    .unwrap()
                    .root_view_controller()
                    .view_mut()
                    .safe_area_insets();

                let _ = channel.try_send(SafeAreaInsets::new(
                    insets.top as _,
                    insets.right as _,
                    insets.bottom as _,
                    insets.left as _,
                ));

                layout.set_style(ViewStyle {
                    position: Position::Relative(Relative {
                        flex_shrink: 0.0,
                        flex_grow: 0.0,
                        ..Default::default()
                    }),
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: Dimension::Points(frame.size.width as _),
                        height: Dimension::Points(frame.size.height as _),
                    },
                    ..Default::default()
                });

                layout.compute(Some((frame.size.width as _, frame.size.height as _)));
            });
        });

        Element::builtin(
            Key::new(()),
            Builtin::Window,
            Element::context(
                Key::new(()),
                Rc::new(
                    insets
                        .to_owned()
                        .unwrap_or(SafeAreaInsets::new(20.0, 0.0, 0.0, 0.0)),
                ),
                manager.children(),
            ),
            Some(reference),
        )
    }
}
