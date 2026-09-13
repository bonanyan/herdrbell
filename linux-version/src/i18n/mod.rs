use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppLanguage {
    code: &'static str,
    native_name: &'static str,
}

impl AppLanguage {
    pub const ENGLISH: Self = Self { code: "en", native_name: "English" };
    pub const KOREAN: Self = Self { code: "ko", native_name: "한국어" };
    pub const CHINESE: Self = Self { code: "zh-Hans", native_name: "简体中文" };
    pub const VIETNAMESE: Self = Self { code: "vi", native_name: "Tiếng Việt" };
    pub const GERMAN: Self = Self { code: "de", native_name: "Deutsch" };
    pub const HINDI: Self = Self { code: "hi", native_name: "हिन्दी" };
    pub const FRENCH: Self = Self { code: "fr", native_name: "Français" };
    pub const JAPANESE: Self = Self { code: "ja", native_name: "日本語" };

    pub fn all() -> Vec<Self> {
        vec![
            Self::ENGLISH,
            Self::KOREAN,
            Self::CHINESE,
            Self::VIETNAMESE,
            Self::GERMAN,
            Self::HINDI,
            Self::FRENCH,
            Self::JAPANESE,
        ]
    }

    pub fn code(&self) -> &str {
        self.code
    }

    pub fn native_name(&self) -> &str {
        self.native_name
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::all().into_iter().find(|l| l.code == code)
    }
}

pub struct LocalizationManager {
    language: Mutex<AppLanguage>,
}

impl LocalizationManager {
    pub fn new(initial: AppLanguage) -> Self {
        Self {
            language: Mutex::new(initial),
        }
    }

    pub fn current_language(&self) -> AppLanguage {
        *self.language.lock().unwrap()
    }

    pub fn set_language(&self, lang: AppLanguage) {
        *self.language.lock().unwrap() = lang;
    }

    pub fn string(&self, key: &str, args: &[&str]) -> String {
        let lang = *self.language.lock().unwrap();
        let template = lookup(lang.code, key);
        if args.is_empty() {
            template
        } else {
            let mut result = template;
            for arg in args.iter() {
                result = result.replacen("%@", arg, 1);
            }
            result
        }
    }
}

fn lookup(lang: &str, key: &str) -> String {
    let strings = strings_for_lang(lang);
    if let Some(&s) = strings.get(key) {
        return s.to_string();
    }
    if lang != "en" {
        if let Some(&s) = strings_for_lang("en").get(key) {
            return s.to_string();
        }
    }
    key.to_string()
}

fn strings_for_lang(lang: &str) -> &'static HashMap<&'static str, &'static str> {
    use std::sync::OnceLock;

    static EN: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    static KO: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    static ZH: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    static VI: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    static DE: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    static HI: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    static FR: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    static JA: OnceLock<HashMap<&str, &str>> = OnceLock::new();

    match lang {
        "en" => EN.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("icon.scheme.classic", "Classic");
            m.insert("icon.scheme.custom", "Custom");
            m.insert("menu.configure", "Configure…");
            m.insert("menu.connecting", "Connecting…");
            m.insert("menu.noAgents", "No agents");
            m.insert("menu.noSessions", "No herdr sessions found");
            m.insert("menu.quit", "Quit HerdrBell");
            m.insert("notification.blocked.title", "Agent blocked: %@");
            m.insert("notification.body.session", "session %@");
            m.insert("notification.done.title", "Agent done: %@");
            m.insert("settings.iconStyle", "Icon style");
            m.insert("settings.iconStyle.help", "Choose the icon set used for agent statuses and the menu bar symbol.");
            m.insert("settings.language", "Language");
            m.insert("settings.language.help", "Choose the display language for menus, settings, and notifications. Applies immediately.");
            m.insert("settings.launchAtLogin", "Launch HerdrBell at login");
            m.insert("settings.notifications", "Notify when an agent becomes blocked or done");
            m
        }),
        "ko" => KO.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("icon.scheme.classic", "클래식");
            m.insert("icon.scheme.custom", "사용자 지정");
            m.insert("menu.configure", "설정…");
            m.insert("menu.connecting", "연결 중…");
            m.insert("menu.noAgents", "에이전트 없음");
            m.insert("menu.noSessions", "herdr 세션을 찾을 수 없습니다");
            m.insert("menu.quit", "HerdrBell 종료");
            m.insert("notification.blocked.title", "에이전트 차단됨: %@");
            m.insert("notification.body.session", "세션 %@");
            m.insert("notification.done.title", "에이전트 완료: %@");
            m.insert("settings.iconStyle", "아이콘 스타일");
            m.insert("settings.iconStyle.help", "에이전트 상태와 메뉴 막대 기호에 사용할 아이콘 세트를 선택하세요.");
            m.insert("settings.language", "언어");
            m.insert("settings.language.help", "메뉴, 설정, 알림에 사용할 표시 언어를 선택하세요. 즉시 적용됩니다.");
            m.insert("settings.launchAtLogin", "로그인할 때 HerdrBell 실행");
            m.insert("settings.notifications", "에이전트가 차단되거나 완료되면 알림 보내기");
            m
        }),
        "zh-Hans" => ZH.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("icon.scheme.classic", "经典");
            m.insert("icon.scheme.custom", "自定义");
            m.insert("menu.configure", "设置…");
            m.insert("menu.connecting", "正在连接…");
            m.insert("menu.noAgents", "没有代理");
            m.insert("menu.noSessions", "未找到 herdr 会话");
            m.insert("menu.quit", "退出 HerdrBell");
            m.insert("notification.blocked.title", "代理被阻塞：%@");
            m.insert("notification.body.session", "会话 %@");
            m.insert("notification.done.title", "代理已完成：%@");
            m.insert("settings.iconStyle", "图标样式");
            m.insert("settings.iconStyle.help", "选择用于代理状态和菜单栏图标的图标集。");
            m.insert("settings.language", "语言");
            m.insert("settings.language.help", "选择菜单、设置和通知的显示语言，立即生效。");
            m.insert("settings.launchAtLogin", "登录时启动 HerdrBell");
            m.insert("settings.notifications", "当代理被阻塞或完成时发送通知");
            m
        }),
        "vi" => VI.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("icon.scheme.classic", "Cổ điển");
            m.insert("icon.scheme.custom", "Tùy chỉnh");
            m.insert("menu.configure", "Cấu hình…");
            m.insert("menu.connecting", "Đang kết nối…");
            m.insert("menu.noAgents", "Không có agent");
            m.insert("menu.noSessions", "Không tìm thấy phiên herdr");
            m.insert("menu.quit", "Thoát HerdrBell");
            m.insert("notification.blocked.title", "Agent bị chặn: %@");
            m.insert("notification.body.session", "phiên %@");
            m.insert("notification.done.title", "Agent đã hoàn tất: %@");
            m.insert("settings.iconStyle", "Kiểu biểu tượng");
            m.insert("settings.iconStyle.help", "Chọn bộ biểu tượng dùng cho trạng thái agent và biểu tượng trên thanh menu.");
            m.insert("settings.language", "Ngôn ngữ");
            m.insert("settings.language.help", "Chọn ngôn ngữ hiển thị cho menu, cài đặt và thông báo. Áp dụng ngay lập tức.");
            m.insert("settings.launchAtLogin", "Khởi chạy HerdrBell khi đăng nhập");
            m.insert("settings.notifications", "Thông báo khi agent bị chặn hoặc hoàn tất");
            m
        }),
        "de" => DE.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("icon.scheme.classic", "Klassisch");
            m.insert("icon.scheme.custom", "Eigene");
            m.insert("menu.configure", "Konfigurieren…");
            m.insert("menu.connecting", "Verbinden…");
            m.insert("menu.noAgents", "Keine Agenten");
            m.insert("menu.noSessions", "Keine herdr-Sitzungen gefunden");
            m.insert("menu.quit", "HerdrBell beenden");
            m.insert("notification.blocked.title", "Agent blockiert: %@");
            m.insert("notification.body.session", "Sitzung %@");
            m.insert("notification.done.title", "Agent fertig: %@");
            m.insert("settings.iconStyle", "Symbolstil");
            m.insert("settings.iconStyle.help", "Wählen Sie den Symbolsatz für Agentenstatus und das Symbol in der Menüleiste.");
            m.insert("settings.language", "Sprache");
            m.insert("settings.language.help", "Wählen Sie die Anzeigesprache für Menüs, Einstellungen und Mitteilungen. Wird sofort übernommen.");
            m.insert("settings.launchAtLogin", "HerdrBell bei der Anmeldung starten");
            m.insert("settings.notifications", "Mitteilung senden, wenn ein Agent blockiert oder fertig ist");
            m
        }),
        "hi" => HI.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("icon.scheme.classic", "क्लासिक");
            m.insert("icon.scheme.custom", "कस्टम");
            m.insert("menu.configure", "कॉन्फ़िगर करें…");
            m.insert("menu.connecting", "कनेक्ट हो रहा है…");
            m.insert("menu.noAgents", "कोई एजेंट नहीं");
            m.insert("menu.noSessions", "कोई herdr सत्र नहीं मिला");
            m.insert("menu.quit", "HerdrBell बंद करें");
            m.insert("notification.blocked.title", "एजेंट अवरुद्ध: %@");
            m.insert("notification.body.session", "सत्र %@");
            m.insert("notification.done.title", "एजेंट पूर्ण: %@");
            m.insert("settings.iconStyle", "आइकन शैली");
            m.insert("settings.iconStyle.help", "एजेंट स्थितियों और मेनू बार प्रतीक के लिए आइकन सेट चुनें।");
            m.insert("settings.language", "भाषा");
            m.insert("settings.language.help", "मेन्यू, सेटिंग्स और सूचनाओं के लिए प्रदर्शन भाषा चुनें। तुरंत लागू होती है।");
            m.insert("settings.launchAtLogin", "लॉगइन पर HerdrBell चालू करें");
            m.insert("settings.notifications", "जब कोई एजेंट अवरुद्ध या पूर्ण हो तो सूचित करें");
            m
        }),
        "fr" => FR.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("icon.scheme.classic", "Classique");
            m.insert("icon.scheme.custom", "Personnalisé");
            m.insert("menu.configure", "Configurer…");
            m.insert("menu.connecting", "Connexion…");
            m.insert("menu.noAgents", "Aucun agent");
            m.insert("menu.noSessions", "Aucune session herdr trouvée");
            m.insert("menu.quit", "Quitter HerdrBell");
            m.insert("notification.blocked.title", "Agent bloqué : %@");
            m.insert("notification.body.session", "session %@");
            m.insert("notification.done.title", "Agent terminé : %@");
            m.insert("settings.iconStyle", "Style d'icônes");
            m.insert("settings.iconStyle.help", "Choisissez le jeu d'icônes utilisé pour les états des agents et le symbole de la barre des menus.");
            m.insert("settings.language", "Langue");
            m.insert("settings.language.help", "Choisissez la langue d'affichage des menus, des réglages et des notifications. S'applique immédiatement.");
            m.insert("settings.launchAtLogin", "Lancer HerdrBell à la connexion");
            m.insert("settings.notifications", "Notifier lorsqu'un agent est bloqué ou terminé");
            m
        }),
        "ja" => JA.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("icon.scheme.classic", "クラシック");
            m.insert("icon.scheme.custom", "カスタム");
            m.insert("menu.configure", "設定…");
            m.insert("menu.connecting", "接続中…");
            m.insert("menu.noAgents", "エージェントなし");
            m.insert("menu.noSessions", "herdr セッションが見つかりません");
            m.insert("menu.quit", "HerdrBell を終了");
            m.insert("notification.blocked.title", "エージェントがブロック中：%@");
            m.insert("notification.body.session", "セッション %@");
            m.insert("notification.done.title", "エージェントが完了：%@");
            m.insert("settings.iconStyle", "アイコンスタイル");
            m.insert("settings.iconStyle.help", "エージェントの状態とメニューバーのシンボルに使うアイコンセットを選択します。");
            m.insert("settings.language", "言語");
            m.insert("settings.language.help", "メニュー・設定・通知の表示言語を選択します。すぐに反映されます。");
            m.insert("settings.launchAtLogin", "ログイン時に HerdrBell を起動する");
            m.insert("settings.notifications", "エージェントがブロックまたは完了したときに通知する");
            m
        }),
        _ => strings_for_lang("en"),
    }
}
