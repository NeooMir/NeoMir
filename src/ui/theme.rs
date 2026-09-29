#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeId {
    VsCode,
    RustRover,
    Blue,
    AdwaitaDark,
    KuMirLight,
}

impl ThemeId {
    pub fn all() -> &'static [ThemeId] {
        &[
            ThemeId::VsCode,
            ThemeId::RustRover,
            ThemeId::Blue,
            ThemeId::AdwaitaDark,
            ThemeId::KuMirLight,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ThemeId::VsCode => "VS Code (Dark+)",
            ThemeId::RustRover => "RustRover / Darcula",
            ThemeId::Blue => "Синяя (Cobalt2 / Solarized)",
            ThemeId::AdwaitaDark => "Классическая тёмная (Adwaita)",
            ThemeId::KuMirLight => "Классическая светлая (КуМир)",
        }
    }

    pub fn is_dark(&self) -> bool {
        *self != ThemeId::KuMirLight
    }

    pub fn editor_bg(&self) -> &'static str {
        match self {
            ThemeId::VsCode => "#1e1e1e",
            ThemeId::RustRover => "#1e1f22",
            ThemeId::Blue => "#132230",
            ThemeId::AdwaitaDark => "#242424",
            ThemeId::KuMirLight => "#ffffff",
        }
    }

    pub fn editor_fg(&self) -> &'static str {
        match self {
            ThemeId::VsCode => "#d4d4d4",
            ThemeId::RustRover => "#bcbec4",
            ThemeId::Blue => "#e2ecf7",
            ThemeId::AdwaitaDark => "#f6f5f4",
            ThemeId::KuMirLight => "#2e3436",
        }
    }

    pub fn gutter_bg(&self) -> &'static str {
        match self {
            ThemeId::VsCode => "#181818",
            ThemeId::RustRover => "#1b1c1e",
            ThemeId::Blue => "#0d1721",
            ThemeId::AdwaitaDark => "#1e1e1e",
            ThemeId::KuMirLight => "#f6f6f6",
        }
    }

    pub fn gutter_fg(&self) -> &'static str {
        match self {
            ThemeId::VsCode => "#858585",
            ThemeId::RustRover => "#5a5d63",
            ThemeId::Blue => "#5b7794",
            ThemeId::AdwaitaDark => "#77767b",
            ThemeId::KuMirLight => "#888a85",
        }
    }

    pub fn keyword_color(&self) -> &'static str {
        match self {
            ThemeId::VsCode => "#569cd6",
            ThemeId::RustRover => "#cf8e6d",
            ThemeId::Blue => "#ff9d00",
            ThemeId::AdwaitaDark => "#78aeed",
            ThemeId::KuMirLight => "#0000d6",
        }
    }

    pub fn robot_color(&self) -> &'static str {
        match self {
            ThemeId::VsCode => "#4ec9b0",
            ThemeId::RustRover => "#56a8f5",
            ThemeId::Blue => "#38d878",
            ThemeId::AdwaitaDark => "#8ff0a4",
            ThemeId::KuMirLight => "#008000",
        }
    }

    pub fn sensor_color(&self) -> &'static str {
        match self {
            ThemeId::VsCode => "#dcdcaa",
            ThemeId::RustRover => "#b3ae60",
            ThemeId::Blue => "#00e8c6",
            ThemeId::AdwaitaDark => "#ffa348",
            ThemeId::KuMirLight => "#c15f00",
        }
    }

    pub fn string_color(&self) -> &'static str {
        match self {
            ThemeId::VsCode => "#ce9178",
            ThemeId::RustRover => "#6aab73",
            ThemeId::Blue => "#3ad900",
            ThemeId::AdwaitaDark => "#f66151",
            ThemeId::KuMirLight => "#a40000",
        }
    }

    pub fn comment_color(&self) -> &'static str {
        match self {
            ThemeId::VsCode => "#6a9955",
            ThemeId::RustRover => "#7a7e85",
            ThemeId::Blue => "#0088ff",
            ThemeId::AdwaitaDark => "#9a9996",
            ThemeId::KuMirLight => "#75507b",
        }
    }

    pub fn number_color(&self) -> &'static str {
        match self {
            ThemeId::VsCode => "#b5cea8",
            ThemeId::RustRover => "#2aacb8",
            ThemeId::Blue => "#ff628c",
            ThemeId::AdwaitaDark => "#dc8add",
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
