use std::sync::Arc;

use cpp::cpp;
use cstr::cstr;
use qmetaobject::{QmlEngine, qml_register_singleton_instance, qrc, queued_callback};
use qttypes::{QString, QUrl, QVariant};
use tokio::sync::{
    RwLock,
    mpsc::{self, Receiver, Sender},
    oneshot,
};

use crate::{
    ActionEvent,
    config::Config,
    ui::{
        messagebox::{MessageBoxHelper, MessageBoxResult},
        settings::Settings,
    },
};

mod messagebox;
mod settings;

cpp! {{
    #include <QTranslator>
    #include <QQmlEngine>
    #include <QCoreApplication>
    #include <QtGui/QGuiApplication>
    #include <QtGui/QIcon>
    #include <QQuickStyle>
    #include <QQmlContext>
    #include <QQmlApplicationEngine>
}}

pub struct Ui {
    change_window_visibility: Arc<dyn Fn(bool)>,
    show_message_box: Arc<dyn Fn((QString, QString, QString))>,
    message_box_result_rx: Receiver<MessageBoxResult>,
}

impl Ui {
    pub async fn new(action_event_tx: Sender<ActionEvent>, config: Arc<RwLock<Config>>) -> Self {
        let (message_box_result_tx, message_box_result_rx) = mpsc::channel(8);

        let (settings_cb_tx, settings_cb_rx) = oneshot::channel();
        let (message_box_cb_tx, message_box_cb_rx) = oneshot::channel();
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

            let message_box_helper = MessageBoxHelper::new(message_box_result_tx);
            qml_register_singleton_instance(
                cstr!("MessageBoxHelper"),
                1,
                0,
                cstr!("MessageBoxHelper"),
                message_box_helper,
            );

            engine.load_url(QUrl::from_user_input("qrc:/ui/settings.qml".into()));

            let _ = settings_cb_tx.send(Arc::new({
                let engine_ptr = engine.cpp_ptr();
                queued_callback(move |visible: bool| {
                    cpp!(unsafe [engine_ptr as "QQmlEngine *", visible as "bool"] {
                        QObject* root_object = ((QQmlApplicationEngine*)engine_ptr)->rootObjects().first();
                        QObject* window = root_object->findChild<QObject*>("window");
                        window->setProperty("visible", QVariant::fromValue(visible));
                    });
                })
            }));

            let _ = message_box_cb_tx.send(Arc::new({
                let engine_ptr = engine.cpp_ptr();
                queued_callback(move |params: (QString, QString, QString)| {
                    let icon = QVariant::from(params.0);
                    let title = QVariant::from(params.1);
                    let text = QVariant::from(params.2);

                    cpp!(unsafe [engine_ptr as "QQmlEngine *", icon as "QVariant", title as "QVariant", text as "QVariant"] {
                        QObject* root_object = ((QQmlApplicationEngine*)engine_ptr)->rootObjects().first();
                        QObject* messageBox = root_object->findChild<QObject*>("messageBox");
                        messageBox->setProperty("icon", icon);
                        messageBox->setProperty("title", title);
                        messageBox->setProperty("text", text);
                        messageBox->setProperty("visible", true);
                    });
                })
            }));

            engine.exec();
        });

        let obj = Self {
            change_window_visibility: settings_cb_rx.await.unwrap(),
            show_message_box: message_box_cb_rx.await.unwrap(),
            message_box_result_rx,
        };

        obj
    }

    pub fn open_settings(&self) {
        self.change_window_visibility.as_ref()(true);
    }

    pub async fn show_info(&mut self, title: &str, text: &str) -> MessageBoxResult {
        self.show_message_box.as_ref()((
            QString::from("dialog-information"),
            QString::from(title),
            QString::from(text),
        ));
        self.message_box_result_rx.recv().await.unwrap()
    }

    pub async fn show_error(&mut self, title: &str, text: &str) -> MessageBoxResult {
        self.show_message_box.as_ref()((
            QString::from("dialog-error"),
            QString::from(title),
            QString::from(text),
        ));
        self.message_box_result_rx.recv().await.unwrap()
    }
}

qrc!(settings_ui, "ui" as "ui" {
    "settings.qml",
    "AudioPage.qml",
    "MainPage.qml",
    "MessageBox.qml",
    "components/ConfigLabel.qml",
    "lang/pl_PL.qm",
    "lang/de_DE.qm",
    "lang/fr_FR.qm",
    "lang/es_ES.qm"
});
