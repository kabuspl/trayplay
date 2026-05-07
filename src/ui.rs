use std::sync::Arc;

use cpp::cpp;
use cstr::cstr;
use qmetaobject::{QmlEngine, qml_register_singleton_instance, qrc, queued_callback};
use qttypes::{QString, QUrl, QVariant};
use tokio::sync::{Mutex, RwLock, mpsc::Sender, oneshot};

use crate::{ActionEvent, config::Config, settings::Settings};

cpp! {{
    #include <QTranslator>
    #include <QQmlEngine>
    #include <QCoreApplication>
    #include <QtGui/QGuiApplication>
    #include <QtGui/QIcon>
    #include <QQuickStyle>
    #include <QQmlContext>
}}

pub struct Ui {
    change_window_visibility: Arc<dyn Fn(bool)>,
}

impl Ui {
    pub async fn new(action_event_tx: Sender<ActionEvent>, config: Arc<RwLock<Config>>) -> Self {
        let (settings_cb_tx, settings_cb_rx) = oneshot::channel();
        tokio::spawn(async move {
            let mut engine = QmlEngine::new();

            settings_ui();

            {
                let engine_ptr = engine.cpp_ptr();
                cpp!(unsafe [engine_ptr as "QQmlEngine *"] {
                    QGuiApplication::setWindowIcon(QIcon::fromTheme("media-skip-backward"));
                    QQuickStyle::setStyle(QStringLiteral("org.kde.desktop"));

                    static QTranslator translator;
                    QCoreApplication::removeTranslator(&translator);

                    QString lang_id = QLocale::system().name();

                    if (lang_id != "en") {
                        if (translator.load(":/ui/lang/" + lang_id + ".qm")) {
                            QCoreApplication::installTranslator(&translator);
                        }
                    }

                    engine_ptr->retranslate();
                });
            }

            let settings = Settings::new(config, action_event_tx).await;

            qml_register_singleton_instance(cstr!("Settings"), 1, 0, cstr!("Settings"), settings);

            engine.load_url(QUrl::from_user_input("qrc:/ui/settings.qml".into()));

            let _ = settings_cb_tx.send(Arc::new({
                let engine_ptr = engine.cpp_ptr();
                queued_callback(move |visible: bool| {
                    cpp!(unsafe [engine_ptr as "QQmlEngine *", visible as "bool"] {
                        QQmlContext* qml_context = engine_ptr->rootContext();
                        QObject* window = qml_context->findObjectRecursively("window");
                        window->setProperty("visible", QVariant::fromValue(visible));
                    });
                })
            }));

            engine.exec();
        });

        let obj = Self {
            change_window_visibility: settings_cb_rx.await.unwrap(),
        };

        obj
    }

    pub fn open_settings(&self) {
        self.change_window_visibility.as_ref()(true);
    }
}

qrc!(settings_ui, "ui" as "ui" {
    "settings.qml",
    "AudioPage.qml",
    "MainPage.qml",
    "components/ConfigLabel.qml",
    "lang/pl_PL.qm",
    "lang/de_DE.qm",
    "lang/fr_FR.qm",
    "lang/es_ES.qm"
});
