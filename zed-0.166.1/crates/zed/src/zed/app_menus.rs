use collab_ui::collab_panel;
use gpui::{Menu, MenuItem, OsAction};
use terminal_view::terminal_panel;

pub fn app_menus() -> Vec<Menu> {
    use zed_actions::Quit;

    vec![
        Menu {
            name: "Zed".into(),
            items: vec![
                MenuItem::action("关于Zed…", zed_actions::About),
                MenuItem::action("检查更新", auto_update::Check),
                MenuItem::separator(),
                MenuItem::submenu(Menu {
                    name: "Settings".into(),
                    items: vec![
                        MenuItem::action("打开设置", super::OpenSettings),
                        MenuItem::action("打开按键绑定", zed_actions::OpenKeymap),
                        MenuItem::action("打开默认设置", super::OpenDefaultSettings),
                        MenuItem::action(
                            "打开默认按键绑定",
                            zed_actions::OpenDefaultKeymap,
                        ),
                        MenuItem::action("打开项目设置", super::OpenProjectSettings),
                        MenuItem::action(
                            "选择主题...",
                            zed_actions::theme_selector::Toggle::default(),
                        ),
                    ],
                }),
                MenuItem::separator(),
                MenuItem::submenu(Menu {
                    name: "Services".into(),
                    items: vec![],
                }),
                MenuItem::separator(),
                MenuItem::action("扩展", zed_actions::Extensions),
                MenuItem::action("安装CLI", install_cli::Install),
                MenuItem::separator(),
                MenuItem::action("隐藏Zed", super::Hide),
                MenuItem::action("隐藏其他", super::HideOthers),
                MenuItem::action("显示所有", super::ShowAll),
                MenuItem::action("退出", Quit),
            ],
        },
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("新建", workspace::NewFile),
                MenuItem::action("新建窗口", workspace::NewWindow),
                MenuItem::separator(),
                MenuItem::action("打开…", workspace::Open),
                MenuItem::action(
                    "打开最近...",
                    zed_actions::OpenRecent {
                        create_new_window: true,
                    },
                ),
                MenuItem::separator(),
                MenuItem::action("添加文件夹到项目…", workspace::AddFolderToProject),
                MenuItem::action("保存", workspace::Save { save_intent: None }),
                MenuItem::action("另存为…", workspace::SaveAs),
                MenuItem::action("保存全部", workspace::SaveAll { save_intent: None }),
                MenuItem::action(
                    "关闭编辑器",
                    workspace::CloseActiveItem { save_intent: None },
                ),
                MenuItem::action("关闭窗口", workspace::CloseWindow),
            ],
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::os_action("撤销", editor::actions::Undo, OsAction::Undo),
                MenuItem::os_action("重做", editor::actions::Redo, OsAction::Redo),
                MenuItem::separator(),
                MenuItem::os_action("剪切", editor::actions::Cut, OsAction::Cut),
                MenuItem::os_action("复制", editor::actions::Copy, OsAction::Copy),
                MenuItem::os_action("粘贴", editor::actions::Paste, OsAction::Paste),
                MenuItem::separator(),
                MenuItem::action("查找", search::buffer_search::Deploy::find()),
                MenuItem::action("在项目中查找", workspace::DeploySearch::find()),
                MenuItem::separator(),
                MenuItem::action(
                    "切换行注释",
                    editor::actions::ToggleComments::default(),
                ),
            ],
        },
        Menu {
            name: "Selection".into(),
            items: vec![
                MenuItem::os_action(
                    "选择全部",
                    editor::actions::SelectAll,
                    OsAction::SelectAll,
                ),
                MenuItem::action("扩展选择", editor::actions::SelectLargerSyntaxNode),
                MenuItem::action("收缩选择", editor::actions::SelectSmallerSyntaxNode),
                MenuItem::separator(),
                MenuItem::action("在上方添加光标", editor::actions::AddSelectionAbove),
                MenuItem::action("在下方添加光标", editor::actions::AddSelectionBelow),
                MenuItem::action(
                    "选择下一个匹配项",
                    editor::actions::SelectNext {
                        replace_newest: false,
                    },
                ),
                MenuItem::separator(),
                MenuItem::action("向上移动行", editor::actions::MoveLineUp),
                MenuItem::action("向下移动行", editor::actions::MoveLineDown),
                MenuItem::action("复制选择", editor::actions::DuplicateLineDown),
            ],
        },
        Menu {
            name: "View".into(),
            items: vec![
                MenuItem::action("放大", zed_actions::IncreaseBufferFontSize),
                MenuItem::action("缩小", zed_actions::DecreaseBufferFontSize),
                MenuItem::action("重置缩放", zed_actions::ResetBufferFontSize),
                MenuItem::separator(),
                MenuItem::action("切换左侧面板", workspace::ToggleLeftDock),
                MenuItem::action("切换右侧面板", workspace::ToggleRightDock),
                MenuItem::action("切换底部面板", workspace::ToggleBottomDock),
                MenuItem::action("关闭所有面板", workspace::CloseAllDocks),
                MenuItem::submenu(Menu {
                    name: "Editor Layout".into(),
                    items: vec![
                        MenuItem::action("向上拆分", workspace::SplitUp),
                        MenuItem::action("向下拆分", workspace::SplitDown),
                        MenuItem::action("向左拆分", workspace::SplitLeft),
                        MenuItem::action("向右拆分", workspace::SplitRight),
                    ],
                }),
                MenuItem::separator(),
                MenuItem::action("项目面板", project_panel::ToggleFocus),
                MenuItem::action("大纲面板", outline_panel::ToggleFocus),
                MenuItem::action("协作面板", collab_panel::ToggleFocus),
                MenuItem::action("终端面板", terminal_panel::ToggleFocus),
                MenuItem::separator(),
                MenuItem::action("诊断", diagnostics::Deploy),
                MenuItem::separator(),
            ],
        },
        Menu {
            name: "Go".into(),
            items: vec![
                MenuItem::action("后退", workspace::GoBack),
                MenuItem::action("前进", workspace::GoForward),
                MenuItem::separator(),
                MenuItem::action("命令面板...", zed_actions::command_palette::Toggle),
                MenuItem::separator(),
                MenuItem::action("跳转到文件...", workspace::ToggleFileFinder::default()),
                // MenuItem::action("Go to Symbol in Project", project_symbols::Toggle),
                MenuItem::action("在编辑器中跳转到字符...", editor::actions::ToggleOutline),
                MenuItem::action("转到行/列...", editor::actions::ToggleGoToLine),
                MenuItem::separator(),
                MenuItem::action("转到定义", editor::actions::GoToDefinition),
                MenuItem::action("转到声明", editor::actions::GoToDeclaration),
                MenuItem::action("转到类型定义", editor::actions::GoToTypeDefinition),
                MenuItem::action("查找所有引用", editor::actions::FindAllReferences),
                MenuItem::separator(),
                MenuItem::action("下一个问题", editor::actions::GoToDiagnostic),
                MenuItem::action("上一个问题", editor::actions::GoToPrevDiagnostic),
            ],
        },
        Menu {
            name: "Window".into(),
            items: vec![
                MenuItem::action("最小化", super::Minimize),
                MenuItem::action("缩放", super::Zoom),
                MenuItem::separator(),
            ],
        },
        Menu {
            name: "Help".into(),
            items: vec![
                MenuItem::action("查看遥测数据", zed_actions::OpenTelemetryLog),
                MenuItem::action("查看依赖项许可证", zed_actions::OpenLicenses),
                MenuItem::action("显示欢迎页", workspace::Welcome),
                MenuItem::action("提供反馈...", zed_actions::feedback::GiveFeedback),
                MenuItem::separator(),
                MenuItem::action(
                    "文档",
                    super::OpenBrowser {
                        url: "https://zed.dev/docs".into(),
                    },
                ),
                MenuItem::action(
                    "Zed Twitter",
                    super::OpenBrowser {
                        url: "https://twitter.com/zeddotdev".into(),
                    },
                ),
                MenuItem::action(
                    "加入团队",
                    super::OpenBrowser {
                        url: "https://zed.dev/jobs".into(),
                    },
                ),
            ],
        },
    ]
}
