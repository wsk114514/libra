use std::fs::File;
use tempfile::tempdir;

// 引入你刚才写的命令
use libra::command::stats::{StatsArgs, execute_safe};
use libra::utils::output::OutputConfig;
// 换成了能真正初始化仓库的 setup_with_new_libra_in
use libra::utils::test::{setup_with_new_libra_in, ChangeDirGuard};

#[tokio::test]
async fn test_stats_counts_extensions_in_workdir() {
    // 1. 创建一个临时测试目录，并初始化为真正的 Libra 测试仓库
    let temp = tempdir().unwrap();
    setup_with_new_libra_in(temp.path()).await;
    let _guard = ChangeDirGuard::new(temp.path());

    // 2. 在临时目录里造几个假文件
    File::create(temp.path().join("file1.txt")).unwrap();
    File::create(temp.path().join("file2.txt")).unwrap();
    File::create(temp.path().join("image.png")).unwrap();
    File::create(temp.path().join("no_extension_file")).unwrap();

    // 3. 运行 stats 命令
    let args = StatsArgs { path: None };
    let result = execute_safe(args, &OutputConfig::default()).await;
    
    assert!(result.is_ok(), "stats command should succeed in a valid repository");
}