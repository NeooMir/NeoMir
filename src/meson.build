sources = files(
  'main.rs',
  'robot.rs',
  'lang/mod.rs',
  'lang/token.rs',
  'lang/ast.rs',
  'lang/parser.rs',
  'lang/vm.rs',
  'ui/mod.rs',
  'ui/app_window.rs',
  'ui/editor_view.rs',
  'ui/field_view.rs',
  'ui/console_view.rs',
)

cargo_script = meson.project_source_root() / 'build-aux' / 'cargo.sh'

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  command: [
    cargo_script,
    meson.project_source_root(),
    meson.project_build_root(),
    '@OUTPUT@',
    get_option('buildtype'),
    meson.project_name(),
  ],
)
