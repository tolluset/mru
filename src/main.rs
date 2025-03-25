mod config;
mod git;
mod package;
mod repo;

use anyhow::Result;

fn main() -> Result<()> {
    // Config 로드 테스트
    let config = config::Config::load()?;

    println!("설정 파일 로드 성공!");
    println!("기본 커밋 메시지: {}", config.default_commit_message);
    println!("등록된 레포지토리 수: {}", config.repositories.len());

    // 등록된 레포지토리 출력 및 상태 확인
    if !config.repositories.is_empty() {
        println!("\n등록된 레포지토리 목록:");
        for (i, repo) in config.repositories.iter().enumerate() {
            println!("{}. 경로: {}", i + 1, repo.path);
            println!("   GitHub URL: {}", repo.github_url);

            // Git 상태 확인
            match git::check_status(&repo.path) {
                Ok(has_changes) => {
                    if has_changes {
                        println!("   상태: 변경사항 있음");
                    } else {
                        println!("   상태: 깨끗함");
                    }

                    // 현재 브랜치 표시
                    if let Ok(branch) = git::get_current_branch(&repo.path) {
                        println!("   브랜치: {}", branch);
                    }

                    // 패키지 매니저 감지
                    if let Ok(pkg_manager) = package::detect_package_manager(&repo.path) {
                        println!("   패키지 매니저: {}", pkg_manager);
                    }
                }
                Err(e) => println!("   상태 확인 실패: {}", e),
            }
        }
    } else {
        println!("\n등록된 레포지토리가 없습니다.");
    }

    // 설정 파일 경로 출력
    let config_path = config::get_config_path()?;
    println!("\n설정 파일 경로: {}", config_path.display());

    Ok(())
}
