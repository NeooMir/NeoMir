project('neomir',
  version: '1.1',
  meson_version: '>= 0.59.0',
  license: 'GPL-2.0-or-later',
)

i18n = import('i18n')
gnome = import('gnome')

base_id = 'io.github.neomir.app'
datadir = get_option('prefix') / get_option('datadir')
bindir = get_option('prefix') / get_option('bindir')

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.10.0')
dependency('libadwaita-1', version: '>= 1.5.0')

install_data('LICENSE', install_dir: datadir / 'licenses' / 'neomir')

subdir('data')
subdir('src')

gnome.post_install(
  glib_compile_schemas: false,
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
