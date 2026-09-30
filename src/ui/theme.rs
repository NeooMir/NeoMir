#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeId {
    // Ptyxis standard palettes & classics
    AdwaitaDark,
    AdwaitaLight,
    GnomeDark,
    GnomeLight,
    CatppuccinMocha,
    CatppuccinMacchiato,
    CatppuccinFrappe,
    CatppuccinLatte,
    Dracula,
    Nord,
    SolarizedDark,
    SolarizedLight,
    Monokai,
    OneDark,
    GruvboxDark,
    GruvboxLight,
    TokyoNight,
    AyuDark,
    AyuLight,
    Cobalt2,
    VsCode,
    RustRover,
    KuMirLight,
}

impl ThemeId {
    pub fn all() -> &'static [ThemeId] {
        &[
            ThemeId::AdwaitaDark,
            ThemeId::AdwaitaLight,
            ThemeId::GnomeDark,
            ThemeId::GnomeLight,
            ThemeId::CatppuccinMocha,
            ThemeId::CatppuccinMacchiato,
            ThemeId::CatppuccinFrappe,
            ThemeId::CatppuccinLatte,
            ThemeId::Dracula,
            ThemeId::Nord,
            ThemeId::SolarizedDark,
            ThemeId::SolarizedLight,
            ThemeId::Monokai,
            ThemeId::OneDark,
            ThemeId::GruvboxDark,
            ThemeId::GruvboxLight,
            ThemeId::TokyoNight,
            ThemeId::AyuDark,
            ThemeId::AyuLight,
            ThemeId::Cobalt2,
            ThemeId::VsCode,
            ThemeId::RustRover,
            ThemeId::KuMirLight,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ThemeId::AdwaitaDark => "Ptyxis / Adwaita Dark",
            ThemeId::AdwaitaLight => "Ptyxis / Adwaita Light",
            ThemeId::GnomeDark => "Ptyxis / GNOME Dark",
            ThemeId::GnomeLight => "Ptyxis / GNOME Light",
            ThemeId::CatppuccinMocha => "Ptyxis / Catppuccin Mocha",
            ThemeId::CatppuccinMacchiato => "Ptyxis / Catppuccin Macchiato",
            ThemeId::CatppuccinFrappe => "Ptyxis / Catppuccin Frappé",
            ThemeId::CatppuccinLatte => "Ptyxis / Catppuccin Latte",
            ThemeId::Dracula => "Ptyxis / Dracula",
            ThemeId::Nord => "Ptyxis / Nord",
            ThemeId::SolarizedDark => "Ptyxis / Solarized Dark",
            ThemeId::SolarizedLight => "Ptyxis / Solarized Light",
            ThemeId::Monokai => "Ptyxis / Monokai",
            ThemeId::OneDark => "Ptyxis / One Dark",
            ThemeId::GruvboxDark => "Ptyxis / Gruvbox Dark",
            ThemeId::GruvboxLight => "Ptyxis / Gruvbox Light",
            ThemeId::TokyoNight => "Ptyxis / Tokyo Night",
            ThemeId::AyuDark => "Ptyxis / Ayu Dark",
            ThemeId::AyuLight => "Ptyxis / Ayu Light",
            ThemeId::Cobalt2 => "Ptyxis / Cobalt2",
            ThemeId::VsCode => "VS Code (Dark+)",
            ThemeId::RustRover => "RustRover / Darcula",
            ThemeId::KuMirLight => "КуМир (Классическая светлая)",
        }
    }

    pub fn is_dark(&self) -> bool {
        !matches!(
            self,
            ThemeId::AdwaitaLight
                | ThemeId::GnomeLight
                | ThemeId::CatppuccinLatte
                | ThemeId::SolarizedLight
                | ThemeId::GruvboxLight
                | ThemeId::AyuLight
                | ThemeId::KuMirLight
        )
    }

    pub fn editor_bg(&self) -> &'static str {
        match self {
            ThemeId::AdwaitaDark => "#242424",
            ThemeId::AdwaitaLight => "#fafafa",
            ThemeId::GnomeDark => "#1e1e1e",
            ThemeId::GnomeLight => "#ffffff",
            ThemeId::CatppuccinMocha => "#1e1e2e",
            ThemeId::CatppuccinMacchiato => "#24273a",
            ThemeId::CatppuccinFrappe => "#303446",
            ThemeId::CatppuccinLatte => "#eff1f5",
            ThemeId::Dracula => "#282a36",
            ThemeId::Nord => "#2e3440",
            ThemeId::SolarizedDark => "#002b36",
            ThemeId::SolarizedLight => "#fdf6e3",
            ThemeId::Monokai => "#272822",
            ThemeId::OneDark => "#282c34",
            ThemeId::GruvboxDark => "#282828",
            ThemeId::GruvboxLight => "#fbf1c7",
            ThemeId::TokyoNight => "#1a1b26",
            ThemeId::AyuDark => "#0b0e14",
            ThemeId::AyuLight => "#fafafa",
            ThemeId::Cobalt2 => "#132230",
            ThemeId::VsCode => "#1e1e1e",
            ThemeId::RustRover => "#1e1f22",
            ThemeId::KuMirLight => "#ffffff",
        }
    }

    pub fn editor_fg(&self) -> &'static str {
        match self {
            ThemeId::AdwaitaDark => "#f6f5f4",
            ThemeId::AdwaitaLight => "#2e3436",
            ThemeId::GnomeDark => "#ffffff",
            ThemeId::GnomeLight => "#000000",
            ThemeId::CatppuccinMocha => "#cdd6f4",
            ThemeId::CatppuccinMacchiato => "#cad3f5",
            ThemeId::CatppuccinFrappe => "#c6d0f5",
            ThemeId::CatppuccinLatte => "#4c4f69",
            ThemeId::Dracula => "#f8f8f2",
            ThemeId::Nord => "#d8dee9",
            ThemeId::SolarizedDark => "#839496",
            ThemeId::SolarizedLight => "#657b83",
            ThemeId::Monokai => "#f8f8f2",
            ThemeId::OneDark => "#abb2bf",
            ThemeId::GruvboxDark => "#ebdbb2",
            ThemeId::GruvboxLight => "#3c3836",
            ThemeId::TokyoNight => "#c0caf5",
            ThemeId::AyuDark => "#b3b1ad",
            ThemeId::AyuLight => "#575f66",
            ThemeId::Cobalt2 => "#e2ecf7",
            ThemeId::VsCode => "#d4d4d4",
            ThemeId::RustRover => "#bcbec4",
            ThemeId::KuMirLight => "#2e3436",
        }
    }

    pub fn gutter_bg(&self) -> &'static str {
        match self {
            ThemeId::AdwaitaDark => "#1e1e1e",
            ThemeId::AdwaitaLight => "#f0f0f0",
            ThemeId::GnomeDark => "#181818",
            ThemeId::GnomeLight => "#f5f5f5",
            ThemeId::CatppuccinMocha => "#181825",
            ThemeId::CatppuccinMacchiato => "#1e2030",
            ThemeId::CatppuccinFrappe => "#292c3c",
            ThemeId::CatppuccinLatte => "#e6e9ef",
            ThemeId::Dracula => "#21222c",
            ThemeId::Nord => "#242933",
            ThemeId::SolarizedDark => "#073642",
            ThemeId::SolarizedLight => "#eee8d5",
            ThemeId::Monokai => "#1e1f1c",
            ThemeId::OneDark => "#21252b",
            ThemeId::GruvboxDark => "#1d2021",
            ThemeId::GruvboxLight => "#ebdbb2",
            ThemeId::TokyoNight => "#16161e",
            ThemeId::AyuDark => "#06080a",
            ThemeId::AyuLight => "#ededed",
            ThemeId::Cobalt2 => "#0d1721",
            ThemeId::VsCode => "#181818",
            ThemeId::RustRover => "#1b1c1e",
            ThemeId::KuMirLight => "#f6f6f6",
        }
    }

    pub fn gutter_fg(&self) -> &'static str {
        match self {
            ThemeId::AdwaitaDark => "#77767b",
            ThemeId::AdwaitaLight => "#9a9996",
            ThemeId::GnomeDark => "#808080",
            ThemeId::GnomeLight => "#808080",
            ThemeId::CatppuccinMocha => "#6c7086",
            ThemeId::CatppuccinMacchiato => "#6e738d",
            ThemeId::CatppuccinFrappe => "#737994",
            ThemeId::CatppuccinLatte => "#9ca0b0",
            ThemeId::Dracula => "#6272a4",
            ThemeId::Nord => "#4c566a",
            ThemeId::SolarizedDark => "#586e75",
            ThemeId::SolarizedLight => "#93a1a1",
            ThemeId::Monokai => "#75715e",
            ThemeId::OneDark => "#5c6370",
            ThemeId::GruvboxDark => "#7c6f64",
            ThemeId::GruvboxLight => "#a89984",
            ThemeId::TokyoNight => "#565f89",
            ThemeId::AyuDark => "#4f5561",
            ThemeId::AyuLight => "#abb0b6",
            ThemeId::Cobalt2 => "#5b7794",
            ThemeId::VsCode => "#858585",
            ThemeId::RustRover => "#5a5d63",
            ThemeId::KuMirLight => "#888a85",
        }
    }

    pub fn keyword_color(&self) -> &'static str {
        match self {
            ThemeId::AdwaitaDark => "#78aeed",
            ThemeId::AdwaitaLight => "#1c71d8",
            ThemeId::GnomeDark => "#3584e4",
            ThemeId::GnomeLight => "#1c71d8",
            ThemeId::CatppuccinMocha => "#cba6f7",
            ThemeId::CatppuccinMacchiato => "#c6a0f6",
            ThemeId::CatppuccinFrappe => "#ca9ee6",
            ThemeId::CatppuccinLatte => "#8839ef",
            ThemeId::Dracula => "#ff79c6",
            ThemeId::Nord => "#81a1c1",
            ThemeId::SolarizedDark => "#268bd2",
            ThemeId::SolarizedLight => "#268bd2",
            ThemeId::Monokai => "#f92672",
            ThemeId::OneDark => "#c678dd",
            ThemeId::GruvboxDark => "#fb4934",
            ThemeId::GruvboxLight => "#9d0006",
            ThemeId::TokyoNight => "#bb9af7",
            ThemeId::AyuDark => "#ff8f40",
            ThemeId::AyuLight => "#fa8d3e",
            ThemeId::Cobalt2 => "#ff9d00",
            ThemeId::VsCode => "#569cd6",
            ThemeId::RustRover => "#cf8e6d",
            ThemeId::KuMirLight => "#0000d6",
        }
    }

    pub fn robot_color(&self) -> &'static str {
        match self {
            ThemeId::AdwaitaDark => "#8ff0a4",
            ThemeId::AdwaitaLight => "#26a269",
            ThemeId::GnomeDark => "#33d17a",
            ThemeId::GnomeLight => "#26a269",
            ThemeId::CatppuccinMocha => "#a6e3a1",
            ThemeId::CatppuccinMacchiato => "#a6da95",
            ThemeId::CatppuccinFrappe => "#a6d189",
            ThemeId::CatppuccinLatte => "#40a02b",
            ThemeId::Dracula => "#50fa7b",
            ThemeId::Nord => "#a3be8c",
            ThemeId::SolarizedDark => "#859900",
            ThemeId::SolarizedLight => "#859900",
            ThemeId::Monokai => "#a6e22e",
            ThemeId::OneDark => "#98c379",
            ThemeId::GruvboxDark => "#b8bb26",
            ThemeId::GruvboxLight => "#79740e",
            ThemeId::TokyoNight => "#73daca",
            ThemeId::AyuDark => "#7fd962",
            ThemeId::AyuLight => "#6cbf43",
            ThemeId::Cobalt2 => "#38d878",
            ThemeId::VsCode => "#4ec9b0",
            ThemeId::RustRover => "#56a8f5",
            ThemeId::KuMirLight => "#008000",
        }
    }

    pub fn sensor_color(&self) -> &'static str {
        match self {
            ThemeId::AdwaitaDark => "#ffa348",
            ThemeId::AdwaitaLight => "#e66100",
            ThemeId::GnomeDark => "#ff7800",
            ThemeId::GnomeLight => "#e66100",
            ThemeId::CatppuccinMocha => "#f9e2af",
            ThemeId::CatppuccinMacchiato => "#eed49f",
            ThemeId::CatppuccinFrappe => "#e5c890",
            ThemeId::CatppuccinLatte => "#df8e1d",
            ThemeId::Dracula => "#8be9fd",
            ThemeId::Nord => "#88c0d0",
            ThemeId::SolarizedDark => "#b58900",
            ThemeId::SolarizedLight => "#b58900",
            ThemeId::Monokai => "#66d9ef",
            ThemeId::OneDark => "#e5c07b",
            ThemeId::GruvboxDark => "#fabd2f",
            ThemeId::GruvboxLight => "#b57614",
            ThemeId::TokyoNight => "#e0af68",
            ThemeId::AyuDark => "#e6b450",
            ThemeId::AyuLight => "#ed9366",
            ThemeId::Cobalt2 => "#00e8c6",
            ThemeId::VsCode => "#dcdcaa",
            ThemeId::RustRover => "#b3ae60",
            ThemeId::KuMirLight => "#c15f00",
        }
    }

    pub fn string_color(&self) -> &'static str {
        match self {
            ThemeId::AdwaitaDark => "#f66151",
            ThemeId::AdwaitaLight => "#c01c28",
            ThemeId::GnomeDark => "#e01b24",
            ThemeId::GnomeLight => "#c01c28",
            ThemeId::CatppuccinMocha => "#f38ba8",
            ThemeId::CatppuccinMacchiato => "#ed8796",
            ThemeId::CatppuccinFrappe => "#e78284",
            ThemeId::CatppuccinLatte => "#d20f39",
            ThemeId::Dracula => "#f1fa8c",
            ThemeId::Nord => "#ebcb8b",
            ThemeId::SolarizedDark => "#2aa198",
            ThemeId::SolarizedLight => "#2aa198",
            ThemeId::Monokai => "#e6db74",
            ThemeId::OneDark => "#98c379",
            ThemeId::GruvboxDark => "#b8bb26",
            ThemeId::GruvboxLight => "#79740e",
            ThemeId::TokyoNight => "#9ece6a",
            ThemeId::AyuDark => "#aad94c",
            ThemeId::AyuLight => "#86b300",
            ThemeId::Cobalt2 => "#3ad900",
            ThemeId::VsCode => "#ce9178",
            ThemeId::RustRover => "#6aab73",
            ThemeId::KuMirLight => "#a40000",
        }
    }

    pub fn comment_color(&self) -> &'static str {
        match self {
            ThemeId::AdwaitaDark => "#9a9996",
            ThemeId::AdwaitaLight => "#77767b",
            ThemeId::GnomeDark => "#77767b",
            ThemeId::GnomeLight => "#77767b",
            ThemeId::CatppuccinMocha => "#585b70",
            ThemeId::CatppuccinMacchiato => "#5b6078",
            ThemeId::CatppuccinFrappe => "#626880",
            ThemeId::CatppuccinLatte => "#acb0be",
            ThemeId::Dracula => "#6272a4",
            ThemeId::Nord => "#616e88",
            ThemeId::SolarizedDark => "#657b83",
            ThemeId::SolarizedLight => "#93a1a1",
            ThemeId::Monokai => "#75715e",
            ThemeId::OneDark => "#5c6370",
            ThemeId::GruvboxDark => "#928374",
            ThemeId::GruvboxLight => "#928374",
            ThemeId::TokyoNight => "#565f89",
            ThemeId::AyuDark => "#5c6773",
            ThemeId::AyuLight => "#abb0b6",
            ThemeId::Cobalt2 => "#0088ff",
            ThemeId::VsCode => "#6a9955",
            ThemeId::RustRover => "#7a7e85",
            ThemeId::KuMirLight => "#75507b",
        }
    }

    pub fn number_color(&self) -> &'static str {
        match self {
            ThemeId::AdwaitaDark => "#dc8add",
            ThemeId::AdwaitaLight => "#9141ac",
            ThemeId::GnomeDark => "#c061cb",
            ThemeId::GnomeLight => "#9141ac",
            ThemeId::CatppuccinMocha => "#fab387",
            ThemeId::CatppuccinMacchiato => "#f5a97f",
            ThemeId::CatppuccinFrappe => "#ef9f76",
            ThemeId::CatppuccinLatte => "#fe640b",
            ThemeId::Dracula => "#bd93f9",
            ThemeId::Nord => "#b48ead",
            ThemeId::SolarizedDark => "#d33682",
            ThemeId::SolarizedLight => "#d33682",
            ThemeId::Monokai => "#ae81ff",
            ThemeId::OneDark => "#d19a66",
            ThemeId::GruvboxDark => "#d3869b",
            ThemeId::GruvboxLight => "#8f3f71",
            ThemeId::TokyoNight => "#ff9e64",
            ThemeId::AyuDark => "#d2a6ff",
            ThemeId::AyuLight => "#a37acc",
            ThemeId::Cobalt2 => "#ff628c",
            ThemeId::VsCode => "#b5cea8",
            ThemeId::RustRover => "#2aacb8",
            ThemeId::KuMirLight => "#204a87",
        }
    }

    pub fn current_line_bg(&self) -> &'static str {
        "#ffe082"
    }

    pub fn current_line_fg(&self) -> &'static str {
        "#000000"
    }

    pub fn generate_css(&self) -> String {
        format!(
            r#"
            textview.neomir-editor text, textview.neomir-editor {{
                background-color: {bg};
                color: {fg};
            }}
            textview.neomir-gutter text, textview.neomir-gutter {{
                background-color: {gbg};
                color: {gfg};
            }}
            .vim-normal-badge {{
                background-color: #26a269;
                color: #ffffff;
                font-weight: bold;
                border-radius: 4px;
                padding: 1px 6px;
            }}
            .vim-insert-badge {{
                background-color: #e66100;
                color: #ffffff;
                font-weight: bold;
                border-radius: 4px;
                padding: 1px 6px;
            }}
            "#,
            bg = self.editor_bg(),
            fg = self.editor_fg(),
            gbg = self.gutter_bg(),
            gfg = self.gutter_fg(),
        )
    }
}
