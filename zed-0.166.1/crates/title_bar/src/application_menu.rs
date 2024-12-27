use ui::{prelude::*, ContextMenu, NumericStepper, PopoverMenu, PopoverMenuHandle, Tooltip};

pub struct ApplicationMenu {
    context_menu_handle: PopoverMenuHandle<ContextMenu>,
}

impl ApplicationMenu {
    pub fn new(_: &mut ViewContext<Self>) -> Self {
        Self {
            context_menu_handle: PopoverMenuHandle::default(),
        }
    }
}

impl Render for ApplicationMenu {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        PopoverMenu::new("application-menu")
            .menu(move |cx| {
                ContextMenu::build(cx, move |menu, cx| {
                    menu.header("工作区")
                        .action(
                            "打开命令面板",
                            Box::new(zed_actions::command_palette::Toggle),
                        )
                        .when_some(cx.focused(), |menu, focused| menu.context(focused))
                        .custom_row(move |cx| {
                            h_flex()
                                .gap_2()
                                .w_full()
                                .justify_between()
                                .cursor(gpui::CursorStyle::Arrow)
                                .child(Label::new("编辑器字体大小"))
                                .child(
                                    NumericStepper::new(
                                        "buffer-font-size",
                                        theme::get_buffer_font_size(cx).to_string(),
                                        |_, cx| {
                                            cx.dispatch_action(Box::new(
                                                zed_actions::DecreaseBufferFontSize,
                                            ))
                                        },
                                        |_, cx| {
                                            cx.dispatch_action(Box::new(
                                                zed_actions::IncreaseBufferFontSize,
                                            ))
                                        },
                                    )
                                    .reserve_space_for_reset(true)
                                    .when(
                                        theme::has_adjusted_buffer_font_size(cx),
                                        |stepper| {
                                            stepper.on_reset(|_, cx| {
                                                cx.dispatch_action(Box::new(
                                                    zed_actions::ResetBufferFontSize,
                                                ))
                                            })
                                        },
                                    ),
                                )
                                .into_any_element()
                        })
                        .custom_row(move |cx| {
                            h_flex()
                                .gap_2()
                                .w_full()
                                .justify_between()
                                .cursor(gpui::CursorStyle::Arrow)
                                .child(Label::new("界面字体大小"))
                                .child(
                                    NumericStepper::new(
                                        "ui-font-size",
                                        theme::get_ui_font_size(cx).to_string(),
                                        |_, cx| {
                                            cx.dispatch_action(Box::new(
                                                zed_actions::DecreaseUiFontSize,
                                            ))
                                        },
                                        |_, cx| {
                                            cx.dispatch_action(Box::new(
                                                zed_actions::IncreaseUiFontSize,
                                            ))
                                        },
                                    )
                                    .reserve_space_for_reset(true)
                                    .when(
                                        theme::has_adjusted_ui_font_size(cx),
                                        |stepper| {
                                            stepper.on_reset(|_, cx| {
                                                cx.dispatch_action(Box::new(
                                                    zed_actions::ResetUiFontSize,
                                                ))
                                            })
                                        },
                                    ),
                                )
                                .into_any_element()
                        })
                        .header("项目")
                        .action(
                            "添加文件夹到项目...",
                            Box::new(workspace::AddFolderToProject),
                        )
                        .action("打开新项目...", Box::new(workspace::Open))
                        .action(
                            "打开最近项目...",
                            Box::new(zed_actions::OpenRecent {
                                create_new_window: false,
                            }),
                        )
                        .header("帮助")
                        .action("关于Zed", Box::new(zed_actions::About))
                        .action("欢迎页", Box::new(workspace::Welcome))
                        .link(
                            "文档",
                            Box::new(zed_actions::OpenBrowser {
                                url: "https://zed.dev/docs".into(),
                            }),
                        )
                        .action(
                            "提供反馈",
                            Box::new(zed_actions::feedback::GiveFeedback),
                        )
                        .action("检查升级", Box::new(auto_update::Check))
                        .action("查看遥测数据", Box::new(zed_actions::OpenTelemetryLog))
                        .action(
                            "查看依赖项许可证",
                            Box::new(zed_actions::OpenLicenses),
                        )
                        .separator()
                        .action("退出", Box::new(zed_actions::Quit))
                })
                .into()
            })
            .trigger(
                IconButton::new("application-menu", ui::IconName::Menu)
                    .style(ButtonStyle::Subtle)
                    .icon_size(IconSize::Small)
                    .when(!self.context_menu_handle.is_deployed(), |this| {
                        this.tooltip(|cx| Tooltip::text("打开应用菜单", cx))
                    }),
            )
            .with_handle(self.context_menu_handle.clone())
            .into_any_element()
    }
}
