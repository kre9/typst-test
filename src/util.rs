pub mod fs {
    use std::fs::DirEntry;
    use std::io::ErrorKind;
    use std::path::Path;
    use std::{fs, io};

    fn ignore_subset<T: Default>(
        result: io::Result<T>,
        check: impl FnOnce(&io::Error) -> io::Result<bool>,
    ) -> io::Result<T> {
        match result {
            Err(err) if check(&err)? => Ok(Default::default()),
            x => x,
        }
    }

    pub fn collect_dir_entries<P: AsRef<Path>>(path: P) -> io::Result<Vec<DirEntry>> {
        fs::read_dir(path)?.collect::<io::Result<Vec<DirEntry>>>()
    }

    pub fn ensure_dir<P: AsRef<Path>>(path: P, all: bool) -> io::Result<()> {
        fn inner(path: &Path, all: bool) -> io::Result<()> {
            if all {
                return fs::create_dir_all(path);
            }

            fs::create_dir(path)
        }

        ignore_subset(inner(path.as_ref(), all), |e| {
            Ok(e.kind() == ErrorKind::AlreadyExists)
        })
    }

    pub fn ensure_remove_dir<P: AsRef<Path>>(path: P, all: bool) -> io::Result<()> {
        fn inner(path: &Path, all: bool) -> io::Result<()> {
            if all {
                return fs::remove_dir_all(path);
            }

            fs::remove_dir(path)
        }

        ignore_subset(inner(path.as_ref(), all), |e| {
            Ok(e.kind() == ErrorKind::NotFound
                && path
                    .as_ref()
                    .parent()
                    .map(|p| p.try_exists())
                    .transpose()?
                    .is_some_and(|b| b))
        })
    }

    pub fn ensure_empty_dir<P: AsRef<Path>>(path: P, all: bool) -> io::Result<()> {
        fn inner(path: &Path, all: bool) -> io::Result<()> {
            ensure_remove_dir(path, true)?;
            ensure_dir(path, all)
        }

        inner(path.as_ref(), all)
    }

    pub fn common_ancestor<'a>(p: &'a Path, q: &'a Path) -> Option<&'a Path> {
        let mut paths = [p, q];
        paths.sort_by_key(|p| p.as_os_str().len());
        let [short, long] = paths;

        short
            .ancestors()
            .find(|ancestor| long.starts_with(ancestor))
    }

    pub fn is_ancestor_of<'a>(base: &'a Path, path: &'a Path) -> bool {
        common_ancestor(base, path).is_some_and(|a| a == base)
    }
}

pub mod term {
    use std::fmt::Arguments;
    use std::io;
    use std::io::IsTerminal;

    use termcolor::{ColorChoice, ColorSpec, WriteColor};

    pub fn color_stream(color: clap::ColorChoice, stderr: bool) -> termcolor::StandardStream {
        let choice = match color {
            clap::ColorChoice::Auto => {
                if std::io::stderr().is_terminal() {
                    ColorChoice::Auto
                } else {
                    ColorChoice::Never
                }
            }
            clap::ColorChoice::Always => ColorChoice::Always,
            clap::ColorChoice::Never => ColorChoice::Never,
        };

        if stderr {
            termcolor::StandardStream::stderr(choice)
        } else {
            termcolor::StandardStream::stdout(choice)
        }
    }

    pub fn with_color<W: WriteColor + ?Sized>(
        w: &mut W,
        color: impl FnOnce(&mut ColorSpec) -> &mut ColorSpec,
        fmt: Arguments<'_>,
    ) -> io::Result<()> {
        w.set_color(color(&mut ColorSpec::new()))?;
        w.write_fmt(fmt)?;
        w.set_color(&ColorSpec::new().set_reset(true))?;

        Ok(())
    }
}
