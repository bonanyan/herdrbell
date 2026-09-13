use herdrbell::i18n::{AppLanguage, LocalizationManager};

#[test]
fn test_english_strings() {
    let l10n = LocalizationManager::new(AppLanguage::ENGLISH);
    assert_eq!(l10n.string("menu.quit", &[]), "Quit HerdrBell");
    assert_eq!(l10n.string("menu.configure", &[]), "Configure…");
}

#[test]
fn test_chinese_strings() {
    let l10n = LocalizationManager::new(AppLanguage::CHINESE);
    assert_eq!(l10n.string("menu.quit", &[]), "退出 HerdrBell");
}

#[test]
fn test_japanese_strings() {
    let l10n = LocalizationManager::new(AppLanguage::JAPANESE);
    assert_eq!(l10n.string("menu.quit", &[]), "HerdrBell を終了");
}

#[test]
fn test_korean_strings() {
    let l10n = LocalizationManager::new(AppLanguage::KOREAN);
    assert_eq!(l10n.string("menu.quit", &[]), "HerdrBell 종료");
}

#[test]
fn test_string_with_args() {
    let l10n = LocalizationManager::new(AppLanguage::ENGLISH);
    let result = l10n.string("notification.blocked.title", &["opencode"]);
    assert_eq!(result, "Agent blocked: opencode");
}

#[test]
fn test_string_with_args_chinese() {
    let l10n = LocalizationManager::new(AppLanguage::CHINESE);
    let result = l10n.string("notification.done.title", &["qodercli"]);
    assert_eq!(result, "代理已完成：qodercli");
}

#[test]
fn test_unknown_key_returns_key() {
    let l10n = LocalizationManager::new(AppLanguage::ENGLISH);
    assert_eq!(l10n.string("nonexistent.key", &[]), "nonexistent.key");
}

#[test]
fn test_set_language() {
    let l10n = LocalizationManager::new(AppLanguage::ENGLISH);
    assert_eq!(l10n.string("menu.quit", &[]), "Quit HerdrBell");
    l10n.set_language(AppLanguage::FRENCH);
    assert_eq!(l10n.string("menu.quit", &[]), "Quitter HerdrBell");
}

#[test]
fn test_all_languages() {
    let langs = AppLanguage::all();
    assert_eq!(langs.len(), 8);
}

#[test]
fn test_language_from_code() {
    assert_eq!(AppLanguage::from_code("en"), Some(AppLanguage::ENGLISH));
    assert_eq!(AppLanguage::from_code("ja"), Some(AppLanguage::JAPANESE));
    assert_eq!(AppLanguage::from_code("xx"), None);
}
