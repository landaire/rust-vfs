//! Heap-profile the allocation-heavy `VfsPath` operations with Dhat.
//!
//! ```text
//! cargo run --manifest-path Cargo.toml --release --features dhat-heap \
//!     --example dhat_profile -- read-dir
//! cargo run --manifest-path Cargo.toml --release --features dhat-heap \
//!     --example dhat_profile -- join
//! ```
//!
//! Each invocation writes `dhat-vfs-{mode}.json` in the current directory.

use std::hint::black_box;

use vfs::error::VfsErrorKind;
use vfs::{FileSystem, SeekAndRead, SeekAndWrite, VfsMetadata, VfsPath, VfsResult};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

#[derive(Debug)]
struct Names {
    remaining: usize,
}

impl Iterator for Names {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        (self.remaining > 0).then(|| {
            self.remaining -= 1;
            "an-intentionally-long-child-name-to-avoid-small-string-effects".to_owned()
        })
    }
}

#[derive(Debug)]
struct TestFs;

impl FileSystem for TestFs {
    fn read_dir(&self, _: &str) -> VfsResult<Box<dyn Iterator<Item = String> + Send>> {
        Ok(Box::new(Names { remaining: 10_000 }))
    }

    fn create_dir(&self, _: &str) -> VfsResult<()> {
        unsupported()
    }
    fn open_file(&self, _: &str) -> VfsResult<Box<dyn SeekAndRead + Send>> {
        unsupported()
    }
    fn create_file(&self, _: &str) -> VfsResult<Box<dyn SeekAndWrite + Send>> {
        unsupported()
    }
    fn append_file(&self, _: &str) -> VfsResult<Box<dyn SeekAndWrite + Send>> {
        unsupported()
    }
    fn metadata(&self, _: &str) -> VfsResult<VfsMetadata> {
        unsupported()
    }
    fn exists(&self, _: &str) -> VfsResult<bool> {
        unsupported()
    }
    fn remove_file(&self, _: &str) -> VfsResult<()> {
        unsupported()
    }
    fn remove_dir(&self, _: &str) -> VfsResult<()> {
        unsupported()
    }
}

fn unsupported<T>() -> VfsResult<T> {
    Err(VfsErrorKind::NotSupported.into())
}

fn main() {
    let mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "read-dir".to_owned());
    assert!(
        matches!(mode.as_str(), "read-dir" | "join"),
        "mode must be read-dir or join"
    );

    // Construct this before starting Dhat so the report is limited to the
    // operation under test rather than fixture setup.
    let parent = VfsPath::new(TestFs)
        .join("a-deliberately-long-parent-component/and-another-parent-component")
        .unwrap();
    let file_name = format!("dhat-vfs-{mode}.json");
    let profiler = dhat::Profiler::builder()
        .file_name(&file_name)
        .trim_backtraces(None)
        .build();

    match mode.as_str() {
        "read-dir" => {
            for child in parent.read_dir().unwrap() {
                black_box(child.as_str());
            }
        }
        "join" => {
            for _ in 0..10_000 {
                let joined = parent
                    .join("third-component/fourth-component/fifth-component")
                    .unwrap();
                black_box(joined.as_str());
            }
        }
        _ => unreachable!(),
    }

    drop(profiler);
    eprintln!("wrote {file_name}");
}
