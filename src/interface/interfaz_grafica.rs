use chrono::prelude::*;
use chrono::Duration;
use gio::glib;
use gtk::prelude::*;
use gtk::{
    Builder, Button, ComboBox, ComboBoxText, Entry, Inhibit, Label, ListStore, ProgressBar, Window,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

#[derive(Clone)]
pub struct PaymentData {
    pub date: String,
    pub label: String,
    pub message: String,
    pub amount: String,
}

#[allow(dead_code)]
pub enum ChannelData {
    ReceiveTransactions(Vec<PaymentData>),
    Transactions(Vec<TransactionData>),
    Account(AccountData),
    Payment(SenderPayment),
    Balance(BalanceData),
    EndInterface,
    ProofOfInclusion(ProofOfInclusion),
    ResponseProofOfInclusion(bool),
    RequestDownload,
    DownloadData(DownloadData),
    DownloadDataBlocks(DownloadData),
}

#[derive(Clone)]
pub struct DownloadData {
    pub total_data: f64,
    pub received_data: f64,
}

#[derive(Clone)]
pub struct ProofOfInclusion {
    pub block_hash: String,
    pub transaction_hash: String,
}
#[derive(Clone)]
pub struct BalanceData {
    pub available: String,
    pub pending: String,
    pub inmature: String,
    pub total: String,
}

#[derive(Clone)]
pub struct TransactionData {
    pub status: String,
    pub date: String,
    pub tipe: String,
    pub label: String,
    pub amount: String,
}

#[derive(Clone)]
pub struct ReceiveTransaction {
    pub date: String,
    pub label: String,
    pub message: String,
    pub amount: String,
}

#[derive(Clone)]
pub struct AccountData {
    pub name: String,
    pub address: String,
    pub private_key: String,
}

#[derive(Clone)]
pub struct SenderPayment {
    pub address: String,
    pub own_address: String,
    pub amount: f64,
}

struct TransactionFilter {
    min_amount: String,
    tipe: String,
    label: String,
    date: String,
    start_range_date: String,
    end_range_date: String,
}

struct InterfazFilter {
    gtk_entry_min_amount_ref: gtk::Entry,
    gtk_entry_transaction_type_ref: gtk::Entry,
    gtk_list_store_transactions_ref: gtk::ListStore,
    gtk_entry_transaction_filter_label_ref: gtk::Entry,
    gtk_entry_transaction_date_ref: gtk::Entry,
    gtk_calendar_start_range_ref: gtk::Calendar,
    gtk_calendar_end_range_ref: gtk::Calendar,
}

struct CloseAccountParameters {
    combo_cuentas_ref: ComboBoxText,
    combo_cuentas_vector_ref: Rc<RefCell<Vec<(String, String, String)>>>,
    gtk_list_store_transactions_ref: ListStore,
    gtk_list_recent_transactions_ref: ListStore,
    gtk_list_store_payments_ref: ListStore,
    gtk_label_available_overview_ref: gtk::Label,
    gtk_label_inmature_overview_ref: gtk::Label,
    gtk_label_pending_overview_ref: gtk::Label,
    gtk_label_total_overview_ref: gtk::Label,
    gtk_combo_payments_methods_ref: gtk::ComboBox,
    gtk_combo_date_filter_ref: gtk::ComboBox,
    gtk_combo_type_filter_ref: gtk::ComboBox,
}

struct ComboCuentasVector {
    valores: Rc<RefCell<Vec<(String, String, String)>>>,
}

/// initialize the main window with all its objects
pub fn interfaz(
    sender_interfaz: Sender<ChannelData>,
    receiver_interfaz: gtk::glib::Receiver<ChannelData>,
) {
    let sender_cloned = sender_interfaz;
    let builder = Builder::new();
    builder
        .add_from_string(include_str!("window.glade"))
        .expect("Failed to load glade file");

    let window: Window = builder.object("window1").expect("Failed to find window1");
    let combo_cuentas: ComboBoxText = builder.object("CUENTAS").expect("Failed to find CUENTAS");
    let cerrar_cuentas_boton: Button = builder.object("CERRAR_CUENTAS").unwrap();
    let aceptar_cuenta_boton: Button = builder.object("ACEPTAR_CUENTA").unwrap();
    let info_nombre_cuenta: Label = builder.object("INFO_NOMBRE_CUENTA").unwrap();
    let info_bitcoin_address: Label = builder.object("INFO_BITCOIN_ADDRESS").unwrap();
    let info_private_key: Label = builder.object("INFO_PRIVATE_KEY").unwrap();
    let bitcoin_address: Entry = builder.object("BITCOIN_ADDRESS").unwrap();
    let private_key: Entry = builder.object("PRIVATE_KEY").unwrap();
    let nombre_cuenta: Entry = builder.object("NOMBRE_CUENTA").unwrap();

    let sender_end_interface = sender_cloned.clone();
    window.set_title("La rustiqueta");
    window.connect_delete_event(move |_, _| {
        gtk::main_quit();
        let payment = ChannelData::EndInterface;
        sender_end_interface
            .send(payment)
            .expect("error en send de end_interface de gtk");
        Inhibit(false)
    });

    let gtk_list_store_transactions: gtk::ListStore = builder
        .object("gtkListStoreTransactions")
        .expect("Failed to find gtkListStoreTransactions");
    let gtk_list_recent_transactions: gtk::ListStore =
        builder.object("gtkListStoreRecentTransaction").unwrap();

    let gtk_entry_transaction_type: gtk::Entry = builder
        .object("gtkEntryTransactionType")
        .expect("Failed to find gtkEntryTransactionType");
    let gtk_entry_min_amount: gtk::Entry = builder
        .object("gtkEntryMinAmount")
        .expect("Failed to find gtkEntryMinAmount");
    let gtk_entry_transaction_filter_label: gtk::Entry = builder
        .object("gtkEntryTransactionFilter")
        .expect("Failed to find gtkEntryTransactionFilter");
    let gtk_entry_transaction_date: gtk::Entry = builder
        .object("gtkEntryTransactionDate")
        .expect("Failed to find gtkEntryTransactionDate");
    let gtk_calendar_start_range: gtk::Calendar = builder
        .object("gtkCalendarStartRange")
        .expect("Failed to find gtkCalendarStartRange");
    let gtk_calendar_end_range: gtk::Calendar = builder
        .object("gtkCalendarEndRange")
        .expect("Failed to find gtkCalendarEndRange");

    let gtk_label_available_overview: gtk::Label = builder
        .object("gtkLabelAvailableOverview")
        .expect("Failed to find gtkLabelAvailableOverview");
    let gtk_label_pending_overview: gtk::Label = builder
        .object("gtkLabelPendingOverview")
        .expect("Failed to find gtkLabelPendingOverview");
    let gtk_label_inmature_overview: gtk::Label = builder
        .object("gtkLabelInmatureOverview")
        .expect("Failed to find gtkLabelInmatureOverview");
    let gtk_label_total_overview: gtk::Label = builder
        .object("gtkLabelTotalOverview")
        .expect("Failed to find gtkLabelTotalOverview");

    let gtk_combo_payments_methods: ComboBox = builder
        .object("gtkComboBoxPaymentsMethods")
        .expect("Failed to find gtkComboBoxPaymentsMethods");
    gtk_combo_payments_methods.set_active(Some(0));

    let gtk_combo_date_filter: ComboBox = builder
        .object("gtkComboBoxTransactionDate")
        .expect("Failed to find gtkComboBoxTransactionDate");
    gtk_combo_date_filter.set_active(Some(0));

    let gtk_combo_type_filter: ComboBox = builder
        .object("gtkComboBoxTransactionType")
        .expect("Failed to find gtkComboBoxTransactionType");
    gtk_combo_type_filter.set_active(Some(0));

    let block_id_info: Entry = builder.object("BLOCK_ID_ENTRY").unwrap();
    let transaction_id_info: Entry = builder.object("TRANSACTION_ID_ENTRY").unwrap();
    let button_verification: Button = builder.object("BUTTON_VERIFICATION").unwrap();
    let erase_confirmation: Button = builder.object("ERASE_CONFIRMATION").unwrap();
    let confirmation_label: Label = builder.object("CONFIRMATION_LABEL").unwrap();

    let gtk_list_store_payments: gtk::ListStore = builder
        .object("listStorePaymentHistory")
        .expect("Failed to load listStorePaymentHistory in glade file");

    let gtk_progress_bar_download: gtk::ProgressBar = builder
        .object("gtkProgressBarDownload")
        .expect("Failed to load gtkProgressBarDownload in glade file");

    let gtk_progress_bar_download2: gtk::ProgressBar = builder
        .object("gtkProgressBarDownload2")
        .expect("Failed to load gtkProgressBarDownload in glade file");

    gtk_progress_bar_download.set_fraction(0.0);
    let progress_bar_ref = Rc::new(RefCell::new(gtk_progress_bar_download));

    gtk_progress_bar_download2.set_fraction(0.0);
    let progress_bar_ref2 = Rc::new(RefCell::new(gtk_progress_bar_download2));

    let combo_cuentas_vector = ComboCuentasVector {
        valores: Rc::new(RefCell::new(Vec::new())),
    };

    window.show_all();

    proof_of_inclusion_verifying(
        &erase_confirmation,
        &block_id_info,
        &transaction_id_info,
        &button_verification,
        &confirmation_label,
        &sender_cloned,
    );

    connect_combo_cuentas_changed(
        &combo_cuentas,
        &combo_cuentas_vector,
        &info_nombre_cuenta,
        &info_bitcoin_address,
        &info_private_key,
        &sender_cloned,
    );

    connect_aceptar_cuenta_clicked(
        &aceptar_cuenta_boton,
        &nombre_cuenta,
        &bitcoin_address,
        &private_key,
        &combo_cuentas,
        &combo_cuentas_vector,
        &sender_cloned,
    );

    let close_account_parameters = CloseAccountParameters {
        combo_cuentas_ref: combo_cuentas.clone(),
        combo_cuentas_vector_ref: combo_cuentas_vector.valores.clone(),
        gtk_list_store_transactions_ref: gtk_list_store_transactions.clone(),
        gtk_list_recent_transactions_ref: gtk_list_recent_transactions.clone(),
        gtk_list_store_payments_ref: gtk_list_store_payments.clone(),
        gtk_label_available_overview_ref: gtk_label_available_overview.clone(),
        gtk_label_inmature_overview_ref: gtk_label_inmature_overview.clone(),
        gtk_label_pending_overview_ref: gtk_label_pending_overview.clone(),
        gtk_label_total_overview_ref: gtk_label_total_overview.clone(),
        gtk_combo_payments_methods_ref: gtk_combo_payments_methods,
        gtk_combo_date_filter_ref: gtk_combo_date_filter,
        gtk_combo_type_filter_ref: gtk_combo_type_filter,
    };
    connect_cerrar_cuentas_clicked(&cerrar_cuentas_boton, &close_account_parameters);

    send_transaction(
        &builder,
        sender_cloned,
        &combo_cuentas,
        &combo_cuentas_vector,
    );

    receiver_interfaz.attach(None, move |data| {
        match data {
            ChannelData::Transactions(transactions) => {
                gtk_list_store_transactions.clear();
                add_transactions(&gtk_list_store_transactions, &transactions);
                gtk_list_recent_transactions.clear();
                add_recent_transactions(&gtk_list_recent_transactions, &transactions);

                let interfaz_filter = InterfazFilter {
                    gtk_entry_min_amount_ref: gtk_entry_min_amount.clone(),
                    gtk_entry_transaction_type_ref: gtk_entry_transaction_type.clone(),
                    gtk_list_store_transactions_ref: gtk_list_store_transactions.clone(),
                    gtk_entry_transaction_filter_label_ref: gtk_entry_transaction_filter_label
                        .clone(),
                    gtk_entry_transaction_date_ref: gtk_entry_transaction_date.clone(),
                    gtk_calendar_start_range_ref: gtk_calendar_start_range.clone(),
                    gtk_calendar_end_range_ref: gtk_calendar_end_range.clone(),
                };

                connect_entry_filter_transaction_changed(
                    &gtk_entry_transaction_type,
                    &interfaz_filter,
                );

                connect_entry_filter_transaction_changed(&gtk_entry_min_amount, &interfaz_filter);

                connect_entry_filter_transaction_changed(
                    &gtk_entry_transaction_filter_label,
                    &interfaz_filter,
                );

                connect_entry_filter_transaction_changed(
                    &gtk_entry_transaction_date,
                    &interfaz_filter,
                );

                connect_calendar_filter_transaction_changed(
                    &gtk_calendar_start_range,
                    &interfaz_filter,
                );

                connect_calendar_filter_transaction_changed(
                    &gtk_calendar_end_range,
                    &interfaz_filter,
                );
            }
            ChannelData::ReceiveTransactions(transactions) => {
                gtk_list_store_payments.clear();
                add_payments(&gtk_list_store_payments, &transactions);
            }
            ChannelData::Balance(data) => {
                add_balance(
                    &data,
                    &gtk_label_available_overview,
                    &gtk_label_inmature_overview,
                    &gtk_label_pending_overview,
                    &gtk_label_total_overview,
                );
            }
            ChannelData::ResponseProofOfInclusion(response) => {
                result_proof_of_inclusion(response, &confirmation_label);
            }
            ChannelData::DownloadData(response) => {
                result_download_data(response, &progress_bar_ref);
            }
            ChannelData::DownloadDataBlocks(response) => {
                result_download_data_2(response, &progress_bar_ref2);
            }
            _ => println!("error "),
        };
        glib::Continue(true)
    });

    window.show_all();
}

fn result_download_data_2(response: DownloadData, progress_bar_ref: &Rc<RefCell<ProgressBar>>) {
    let percentaje = response.received_data / response.total_data;
    // let fraction = progress_bar_ref.borrow().fraction() + percentaje;
    progress_bar_ref.borrow_mut().set_fraction(percentaje);
}

fn result_proof_of_inclusion(response: bool, confirmation_label: &Label) {
    let confirmation_label_clone = confirmation_label.clone();
    if !response {
        confirmation_label_clone.set_text("FALSE");
    } else {
        confirmation_label_clone.set_text("TRUE");
    }
}

fn result_download_data(response: DownloadData, progress_bar_ref: &Rc<RefCell<ProgressBar>>) {
    let percentaje = response.received_data / response.total_data;
    // let fraction = progress_bar_ref.borrow().fraction() + percentaje;
    progress_bar_ref.borrow_mut().set_fraction(percentaje);
}

// /// shows in window if the proof of inclusion is true or false

fn proof_of_inclusion_verifying(
    erase_confirmation: &Button,
    block_id_info: &Entry,
    transaction_id_info: &Entry,
    button_verification: &Button,
    confirmation_label: &Label,
    sender_interfaz: &Sender<ChannelData>,
) {
    let block_id_info_clone = block_id_info.clone();
    let transaction_id_info_clone = transaction_id_info.clone();
    let sender_cloned = sender_interfaz.clone();
    button_verification.connect_clicked(move |_| {
        let block_id_entry = block_id_info_clone.text().to_string();
        let transaction_id_entry = transaction_id_info_clone.text().to_string();
        let proof_of_inclusion = ChannelData::ProofOfInclusion(ProofOfInclusion {
            block_hash: block_id_entry,
            transaction_hash: transaction_id_entry,
        });
        sender_cloned
            .send(proof_of_inclusion)
            .expect("Failed of Proof of inclusion");

        println!(
            " INTERFAZ  {}, {}",
            block_id_info_clone.text(),
            transaction_id_info_clone.text()
        );
    });

    let block_id_info_clone = block_id_info.clone();
    let transaction_id_info_clone = transaction_id_info.clone();
    let confirmation_label_clone = confirmation_label.clone();
    erase_confirmation.connect_clicked(move |_| {
        block_id_info_clone.set_text("");
        transaction_id_info_clone.set_text("");
        confirmation_label_clone.set_text("");

        //println!("{}, {}", block_id_entry, transaction_id_entry);
    });
}

/// read the entry and realise a transaction from the user
fn send_transaction(
    builder: &Builder,
    sender_cloned: Sender<ChannelData>,
    combo_cuentas: &ComboBoxText,
    combo_cuentas_vector: &ComboCuentasVector,
) {
    let aceptar_transaction: Button = builder.object("ACEPT_SEND").unwrap();
    let deny_transaction: Button = builder.object("CANCEL_TRANSACTION").unwrap();
    let monto_transaction: gtk::SpinButton = builder.object("MONTO_TRANSACTION").unwrap();
    let bitcoin_address_transaction: Entry = builder.object("ID_TRANSACTION").unwrap();

    let monto_transaction_clone = monto_transaction.clone();

    let bitcoin_address_transaction_clone = bitcoin_address_transaction.clone();
    let clone_combo_cuentas = combo_cuentas.clone();
    let clone_vector = ComboCuentasVector {
        valores: combo_cuentas_vector.valores.clone(),
    };

    aceptar_transaction.connect_clicked(move |_| {
        let bitcoin_address_entry = bitcoin_address_transaction.text().to_string();
        let monto_transaction_entry = monto_transaction_clone.value();
        let monto_rounder = (monto_transaction_entry * 1e8).round() / 1e8;

        println!("{}, {:?}", bitcoin_address_entry, monto_rounder);

        let dato_enviado_desde_gtk = "Intentando enviar dato de pago desde gtk".to_string();
        println!("{}", dato_enviado_desde_gtk);
        let payment = ChannelData::Payment(SenderPayment {
            address: bitcoin_address_entry,
            amount: monto_rounder,
            own_address: get_active_account_address(&clone_combo_cuentas, &clone_vector),
        });
        sender_cloned
            .send(payment)
            .expect("error en send de sender_payment de gtk");

        monto_transaction_clone.set_value(0.0);
        bitcoin_address_transaction.set_text("");
    });
    let monto_transaction_clone = monto_transaction;
    deny_transaction.connect_clicked(move |_| {
        monto_transaction_clone.set_value(0.0);
        bitcoin_address_transaction_clone.set_text("");
    });
}
fn get_active_account_address(
    combo_cuentas: &ComboBoxText,
    combo_cuentas_vector: &ComboCuentasVector,
) -> String {
    let cloned_combo_cuentas_vector = ComboCuentasVector {
        valores: combo_cuentas_vector.valores.clone(),
    };
    let cloned_combo_cuentas = combo_cuentas.clone();
    if let Some(active_text) = cloned_combo_cuentas.active_text() {
        let vector = cloned_combo_cuentas_vector.valores.borrow();
        for cuenta in vector.iter() {
            if cuenta.0 == active_text {
                return cuenta.1.to_string();
            }
        }
    }
    "".to_string()
}
/// check with active_text is in the comboCheckText
fn connect_combo_cuentas_changed(
    combo_cuentas: &ComboBoxText,
    combo_cuentas_vector: &ComboCuentasVector,
    info_nombre_cuenta: &Label,
    info_bitcoin_address: &Label,
    info_private_key: &Label,
    sender_cloned: &Sender<ChannelData>,
) {
    let cloned_combo_cuentas_vector = combo_cuentas_vector.valores.clone();
    let cloned_combo_cuentas = combo_cuentas.clone();
    let cloned_info_nombre_cuenta = info_nombre_cuenta.clone();
    let cloned_info_bitcoin_address = info_bitcoin_address.clone();
    let cloned_info_private_key = info_private_key.clone();
    let cloned_sender = sender_cloned.clone();

    combo_cuentas.connect_changed(move |_| {
        if let Some(active_text) = cloned_combo_cuentas.active_text() {
            update_account_info(
                &active_text,
                &cloned_combo_cuentas_vector.borrow(),
                &cloned_info_nombre_cuenta,
                &cloned_info_bitcoin_address,
                &cloned_info_private_key,
                &cloned_sender,
            );
        } else {
            clear_account_info(
                &cloned_info_nombre_cuenta,
                &cloned_info_bitcoin_address,
                &cloned_info_private_key,
            );
        }
    });
}

/// depending of the active_account, updates the account info
fn update_account_info(
    active_text: &str,
    cuentas: &Vec<(String, String, String)>,
    info_nombre_cuenta: &Label,
    info_bitcoin_address: &Label,
    info_private_key: &Label,
    sender_cloned: &Sender<ChannelData>,
) {
    for cuenta in cuentas {
        if cuenta.0 == active_text {
            info_nombre_cuenta.set_text(&cuenta.0);
            info_bitcoin_address.set_text(&cuenta.1);
            info_private_key.set_text(&cuenta.2);

            let dato = ChannelData::Account(AccountData {
                name: cuenta.0.clone(),
                address: cuenta.1.clone(),
                private_key: cuenta.2.clone(),
            });
            sender_cloned
                .send(dato)
                .expect("Error en send AccountData de gtk");
        }
    }
}

/// clear all the account info
fn clear_account_info(
    info_nombre_cuenta: &Label,
    info_bitcoin_address: &Label,
    info_private_key: &Label,
) {
    info_nombre_cuenta.set_text("");
    info_bitcoin_address.set_text("");
    info_private_key.set_text("");
}

/// if the user click in the button, the account is added in a list of accounts
fn connect_aceptar_cuenta_clicked(
    aceptar_cuenta_boton: &Button,
    nombre_cuenta: &Entry,
    bitcoin_address: &Entry,
    private_key: &Entry,
    combo_cuentas: &ComboBoxText,
    combo_cuentas_vector: &ComboCuentasVector,
    sender_cloned: &Sender<ChannelData>,
) {
    let cloned_combo_cuentas_vector = ComboCuentasVector {
        valores: combo_cuentas_vector.valores.clone(),
    };
    let cloned_combo_cuentas = combo_cuentas.clone();
    let cloned_nombre_cuenta = nombre_cuenta.clone();
    let cloned_bitcoin_address = bitcoin_address.clone();
    let cloned_private_key = private_key.clone();
    let cloned_sender = sender_cloned.clone();

    aceptar_cuenta_boton.connect_clicked(move |_| {
        let cloned_nombre = cloned_nombre_cuenta.clone();
        let cloned_address = cloned_bitcoin_address.clone();
        let cloned_key = cloned_private_key.clone();

        add_account(
            &cloned_nombre,
            &cloned_address,
            &cloned_key,
            &cloned_combo_cuentas_vector,
            &cloned_combo_cuentas,
            &cloned_sender,
        );
        clear_entry_fields(&cloned_nombre, &cloned_address, &cloned_key);
    });
}

/// the user creates de account with all the parameters
fn add_account(
    nombre_cuenta: &Entry,
    bitcoin_address: &Entry,
    private_key: &Entry,
    combo_cuentas_vector: &ComboCuentasVector,
    combo_cuentas: &ComboBoxText,
    sender_cloned: &Sender<ChannelData>,
) {
    let nombre_entry = nombre_cuenta.text().to_string();
    let bitcoin_address_entry = bitcoin_address.text().to_string();
    let private_key_entry = private_key.text().to_string();

    let dato = ChannelData::Account(AccountData {
        name: nombre_entry.clone(),
        address: bitcoin_address_entry.clone(),
        private_key: private_key_entry.clone(),
    });
    sender_cloned
        .send(dato)
        .expect("Error en send AccountData de gtk");

    let cuenta_datos = (
        nombre_entry.clone(),
        bitcoin_address_entry,
        private_key_entry,
    );

    combo_cuentas_vector.valores.borrow_mut().push(cuenta_datos);

    combo_cuentas.append_text(&nombre_entry);
    let model = combo_cuentas.model().unwrap();
    let total_items = model.iter_n_children(None);
    let last_index = total_items - 1;
    combo_cuentas.set_active(Some(last_index as u32));
}

/// clear all the account entry fields
fn clear_entry_fields(nombre_cuenta: &Entry, bitcoin_address: &Entry, private_key: &Entry) {
    nombre_cuenta.set_text("");
    bitcoin_address.set_text("");
    private_key.set_text("");
}
/// close all the accounts and clean the vec
fn connect_cerrar_cuentas_clicked(
    cerrar_cuentas_boton: &Button,
    close_account_parameters: &CloseAccountParameters,
) {
    //let clone_close_account_parameters = close_account_parameters.clone();

    let clone_close_account_parameters = CloseAccountParameters {
        combo_cuentas_ref: close_account_parameters.combo_cuentas_ref.clone(),
        combo_cuentas_vector_ref: close_account_parameters.combo_cuentas_vector_ref.clone(),
        gtk_list_store_transactions_ref: close_account_parameters
            .gtk_list_store_transactions_ref
            .clone(),
        gtk_list_recent_transactions_ref: close_account_parameters
            .gtk_list_recent_transactions_ref
            .clone(),
        gtk_list_store_payments_ref: close_account_parameters.gtk_list_store_payments_ref.clone(),
        gtk_label_available_overview_ref: close_account_parameters
            .gtk_label_available_overview_ref
            .clone(),
        gtk_label_inmature_overview_ref: close_account_parameters
            .gtk_label_inmature_overview_ref
            .clone(),
        gtk_label_pending_overview_ref: close_account_parameters
            .gtk_label_pending_overview_ref
            .clone(),
        gtk_label_total_overview_ref: close_account_parameters
            .gtk_label_total_overview_ref
            .clone(),
        gtk_combo_payments_methods_ref: close_account_parameters
            .gtk_combo_payments_methods_ref
            .clone(),
        gtk_combo_date_filter_ref: close_account_parameters.gtk_combo_date_filter_ref.clone(),
        gtk_combo_type_filter_ref: close_account_parameters.gtk_combo_type_filter_ref.clone(),
    };

    cerrar_cuentas_boton.connect_clicked(move |_| {
        clear_accounts(&clone_close_account_parameters);
    });
}

/// the combo_cuentas_vec is cleared and al the labels set in blank
fn clear_accounts(close_account_parameters: &CloseAccountParameters) {
    let cloned_combo_cuentas_vector = close_account_parameters.combo_cuentas_vector_ref.clone();
    let cloned_combo_cuentas = close_account_parameters.combo_cuentas_ref.clone();
    let cloned_list_store_transactions = close_account_parameters
        .gtk_list_store_transactions_ref
        .clone();
    let cloned_list_recent_transactions = close_account_parameters
        .gtk_list_recent_transactions_ref
        .clone();
    let cloned_list_store_payments = close_account_parameters.gtk_list_store_payments_ref.clone();
    let cloned_label_available = close_account_parameters
        .gtk_label_available_overview_ref
        .clone();
    let cloned_label_inmature = close_account_parameters
        .gtk_label_inmature_overview_ref
        .clone();
    let cloned_label_pending = close_account_parameters
        .gtk_label_pending_overview_ref
        .clone();
    let cloned_label_total = close_account_parameters
        .gtk_label_total_overview_ref
        .clone();
    let cloned_combo_payment_methods = close_account_parameters
        .gtk_combo_payments_methods_ref
        .clone();
    let cloned_combo_date_filter = close_account_parameters.gtk_combo_date_filter_ref.clone();
    let cloned_combo_type_filter = close_account_parameters.gtk_combo_type_filter_ref.clone();

    cloned_combo_cuentas_vector.borrow_mut().clear();
    cloned_combo_cuentas.remove_all();
    cloned_list_store_payments.clear();
    cloned_list_store_transactions.clear();
    cloned_list_recent_transactions.clear();
    cloned_label_available.set_text("");
    cloned_label_inmature.set_text("");
    cloned_label_pending.set_text("");
    cloned_label_total.set_text("");
    cloned_combo_payment_methods.set_active(Some(0));
    cloned_combo_date_filter.set_active(Some(0));
    cloned_combo_type_filter.set_active(Some(0));
}

fn add_payments(gtk_list_store_payments: &ListStore, list_payments: &Vec<PaymentData>) {
    for payment in list_payments {
        add_payment(gtk_list_store_payments, payment);
    }
}

fn add_payment(gtk_list_store_payments: &ListStore, payment: &PaymentData) {
    let iter = gtk_list_store_payments.append();
    gtk_list_store_payments.set_value(&iter, 0, &payment.date.to_value());
    gtk_list_store_payments.set_value(&iter, 1, &payment.label.to_value());
    gtk_list_store_payments.set_value(&iter, 2, &payment.message.to_value());
    gtk_list_store_payments.set_value(&iter, 3, &payment.amount.to_value());
}
fn add_recent_transactions(
    gtk_list_recent_transaction: &ListStore,
    list_transactions: &Vec<TransactionData>,
) {
    let last_five = list_transactions.len().saturating_sub(5);
    for transaction in list_transactions.iter().skip(last_five) {
        add_recent_transaction(gtk_list_recent_transaction, transaction);
    }
}

fn add_recent_transaction(gtk_list_recent_transaction: &ListStore, transaction: &TransactionData) {
    let iter = gtk_list_recent_transaction.append();
    gtk_list_recent_transaction.set_value(&iter, 0, &transaction.date.to_value());
    gtk_list_recent_transaction.set_value(&iter, 2, &transaction.label.to_value());
    gtk_list_recent_transaction.set_value(&iter, 1, &transaction.amount.to_value());
}
fn add_transactions(
    gtk_list_store_transactions: &ListStore,
    list_transactions: &Vec<TransactionData>,
) {
    for transaction in list_transactions {
        add_transaction(gtk_list_store_transactions, transaction);
    }
}

fn add_transaction(gtk_list_store_transactions: &ListStore, transaction: &TransactionData) {
    let iter = gtk_list_store_transactions.append();
    //gtk_list_store_transactions.set_value(&iter, 0, &transaction.status.to_value());
    gtk_list_store_transactions.set_value(&iter, 1, &transaction.date.to_value());
    gtk_list_store_transactions.set_value(&iter, 2, &transaction.tipe.to_value());
    gtk_list_store_transactions.set_value(&iter, 3, &transaction.label.to_value());
    gtk_list_store_transactions.set_value(&iter, 4, &transaction.amount.to_value());
}

fn on_gtk_component_change(
    gtk_entry_min_amount: &gtk::Entry,
    gtk_entry_type_transaction: &gtk::Entry,
    gtk_list_store_transactions: &ListStore,
    gtk_entry_transaction_filter_label: &gtk::Entry,
    gtk_entry_transaction_date: &gtk::Entry,
    gtk_calendar_start_range: &gtk::Calendar,
    gtk_calendar_end_range: &gtk::Calendar,
) {
    let gtk_entry_min_amount_text = gtk_entry_min_amount.text().to_string();
    let gtk_entry_transaction_type_text = gtk_entry_type_transaction.text().to_string();
    let gtk_entry_transaction_filter_label_text =
        gtk_entry_transaction_filter_label.text().to_string();
    let gtk_entry_transaction_date_text = gtk_entry_transaction_date.text().to_string();
    let gtk_calendar_start_range = gtk_calendar_start_range.date();
    let gtk_calendar_end_range = gtk_calendar_end_range.date();

    let month_start = gtk_calendar_start_range.1 + 1;
    let month_end = gtk_calendar_end_range.1 + 1;

    gtk_list_store_transactions.clear();
    let filter = TransactionFilter {
        min_amount: gtk_entry_min_amount_text,
        tipe: gtk_entry_transaction_type_text,
        label: gtk_entry_transaction_filter_label_text,
        date: gtk_entry_transaction_date_text,
        start_range_date: format!(
            "{}/{}/{}",
            gtk_calendar_start_range.2, month_start, gtk_calendar_start_range.0
        ),
        end_range_date: format!(
            "{}/{}/{}",
            gtk_calendar_end_range.2, month_end, gtk_calendar_end_range.0
        ),
    };
    let list_transactions: Vec<TransactionData> = get_transactions_filtered(&filter);
    add_transactions(gtk_list_store_transactions, &list_transactions);
}

fn get_transactions_filtered(filter: &TransactionFilter) -> Vec<TransactionData> {
    let transactions = vec![
        TransactionData {
            status: "1".to_string(),
            date: "15/06/2023 15:18".to_string(),
            tipe: "Mined".to_string(),
            label: "1L9nr4GX4Zmd7gDL1UT75QPUqxSgNTvdHd".to_string(),
            amount: "50.00000000".to_string(),
        },
        TransactionData {
            status: "2".to_string(),
            date: "12/06/2023 15:18".to_string(),
            tipe: "Send to".to_string(),
            label: "4D6nr4GX4Zmd7gDL1UT75QPUqxSgNTvkJl".to_string(),
            amount: "20.00000000".to_string(),
        },
        TransactionData {
            status: "3".to_string(),
            date: "01/06/2023 15:18".to_string(),
            tipe: "Mined".to_string(),
            label: "6R6nr4GX4Zmd7gDL1UT75QPUqxSgNTvoLp".to_string(),
            amount: "40.00000000".to_string(),
        },
        TransactionData {
            status: "4".to_string(),
            date: "20/05/2023 15:18".to_string(),
            tipe: "Mined".to_string(),
            label: "6J0nr4GX4Zmd7gDL1UT75QPUqxSgNTvnOp".to_string(),
            amount: "10.00000000".to_string(),
        },
        TransactionData {
            status: "5".to_string(),
            date: "10/04/2022 15:18".to_string(),
            tipe: "Mined".to_string(),
            label: "6J0nr4GX4Zmd7gDL1UT75QPUqxSgNTvnOp".to_string(),
            amount: "10.00000000".to_string(),
        },
    ];

    let mut filtered_transactions = transactions;

    if !filter.min_amount.is_empty() {
        filtered_transactions.retain(|transaction| {
            let amount: f64 = transaction
                .amount
                .parse()
                .expect("Failed to parse transaction amount in method get_transactions_filtered");
            let min_amount: f64 = filter
                .min_amount
                .parse()
                .expect("Failed to parse filter min amount in method get_transactions_filtered");
            amount >= min_amount
        });
    }

    if !filter.tipe.is_empty() && filter.tipe != "All" {
        filtered_transactions.retain(|transaction| {
            let transaction_type = &transaction.tipe;
            transaction_type == &filter.tipe
        });
    }

    if !filter.label.is_empty() {
        filtered_transactions.retain(|transaction| {
            let transaction_label = &transaction.label;
            transaction_label.contains(&filter.label)
        });
    }

    if !filter.date.is_empty() && filter.date != "All" {
        if filter.date == "Today" {
            let today = Local::now().naive_local().date();
            filtered_transactions.retain(|transaction| {
                let transaction_date = NaiveDateTime::parse_from_str(
                    &transaction.date,
                    "%d/%m/%Y %H:%M",
                )
                .expect(
                    "Failed to parse transaction date in method get_transactions_filtered by ALL",
                );
                transaction_date.date() == today
            });
        } else if filter.date == "This month" {
            let this_month = Local::now().naive_local().month();
            filtered_transactions.retain(|transaction| {
                let transaction_date = NaiveDateTime::parse_from_str(
                    &transaction.date,
                    "%d/%m/%Y %H:%M",
                )
                .expect(
                    "Failed to parse transaction date in method get_transactions_filtered by month",
                );
                transaction_date.month() == this_month
            });
        } else if filter.date == "This year" {
            let this_year = Local::now().naive_local().year();
            filtered_transactions.retain(|transaction| {
                let transaction_date = NaiveDateTime::parse_from_str(
                    &transaction.date,
                    "%d/%m/%Y %H:%M",
                )
                .expect(
                    "Failed to parse transaction date in method get_transactions_filtered by year",
                );
                transaction_date.year() == this_year
            });
        } else if filter.date == "This week" {
            let today = Local::now().naive_local().date();
            let start_of_week =
                today - Duration::days(today.weekday().num_days_from_monday() as i64);
            filtered_transactions.retain(|transaction| {
                let transaction_date = NaiveDateTime::parse_from_str(
                    &transaction.date,
                    "%d/%m/%Y %H:%M",
                )
                .expect(
                    "Failed to parse transaction date in method get_transactions_filtered by week",
                );
                transaction_date.date() >= start_of_week
            });
        } else if filter.date == "Last month" {
            let today = Local::now().naive_local();
            let one_day = Duration::days(1);
            let last_day_of_previous_month = today.with_day(1).unwrap() - one_day;
            let first_day_of_previous_month = last_day_of_previous_month
                .with_day(1)
                .expect("Failed to calculate first day of previous month");
            filtered_transactions.retain(|transaction| {
                                 let transaction_date = NaiveDateTime::parse_from_str(&transaction.date, "%d/%m/%Y %H:%M").expect("Failed to parse transaction date in method get_transactions_filtered by last month");
                                 transaction_date.date() >= first_day_of_previous_month.date() && transaction_date.date() <= last_day_of_previous_month.date()
                             });
        } else if filter.date == "Range" {
            let start_range = NaiveDate::parse_from_str(&filter.start_range_date, "%d/%m/%Y")
                .expect(
                    "Failed to parse start range date in method get_transactions_filtered by range",
                );
            let end_range = NaiveDate::parse_from_str(&filter.end_range_date, "%d/%m/%Y").expect(
                "Failed to parse end range date in method get_transactions_filtered by range",
            );
            filtered_transactions.retain(|transaction| {
                let transaction_date = NaiveDateTime::parse_from_str(
                    &transaction.date,
                    "%d/%m/%Y %H:%M",
                )
                .expect(
                    "Failed to parse transaction date in method get_transactions_filtered by range",
                );
                transaction_date.date() >= start_range && transaction_date.date() <= end_range
            });
        }
    }

    filtered_transactions
}

fn connect_entry_filter_transaction_changed(
    gtk_entry_to_add_method: &gtk::Entry,
    interfaz_filter: &InterfazFilter,
) {
    let cloned_gtk_entry_min_amount = interfaz_filter.gtk_entry_min_amount_ref.clone();
    let cloned_gtk_entry_transaction_type = interfaz_filter.gtk_entry_transaction_type_ref.clone();
    let cloned_gtk_list_store_transactions =
        interfaz_filter.gtk_list_store_transactions_ref.clone();
    let cloned_gtk_entry_transaction_filter_label = interfaz_filter
        .gtk_entry_transaction_filter_label_ref
        .clone();
    let cloned_gtk_entry_transaction_date = interfaz_filter.gtk_entry_transaction_date_ref.clone();
    let cloned_gtk_calendar_start_range = interfaz_filter.gtk_calendar_start_range_ref.clone();
    let cloned_gtk_calendar_end_range = interfaz_filter.gtk_calendar_end_range_ref.clone();

    gtk_entry_to_add_method.connect_changed(move |_| {
        on_gtk_component_change(
            &cloned_gtk_entry_min_amount,
            &cloned_gtk_entry_transaction_type,
            &cloned_gtk_list_store_transactions,
            &cloned_gtk_entry_transaction_filter_label,
            &cloned_gtk_entry_transaction_date,
            &cloned_gtk_calendar_start_range,
            &cloned_gtk_calendar_end_range,
        );
    });
}

fn connect_calendar_filter_transaction_changed(
    gtk_calendar_to_add_method: &gtk::Calendar,
    interfaz_filter: &InterfazFilter,
) {
    let cloned_gtk_entry_min_amount = interfaz_filter.gtk_entry_min_amount_ref.clone();
    let cloned_gtk_entry_transaction_type = interfaz_filter.gtk_entry_transaction_type_ref.clone();
    let cloned_gtk_list_store_transactions =
        interfaz_filter.gtk_list_store_transactions_ref.clone();
    let cloned_gtk_entry_transaction_filter_label = interfaz_filter
        .gtk_entry_transaction_filter_label_ref
        .clone();
    let cloned_gtk_entry_transaction_date = interfaz_filter.gtk_entry_transaction_date_ref.clone();
    let cloned_gtk_calendar_start_range = interfaz_filter.gtk_calendar_start_range_ref.clone();
    let cloned_gtk_calendar_end_range = interfaz_filter.gtk_calendar_end_range_ref.clone();

    gtk_calendar_to_add_method.connect_day_selected(move |_| {
        on_gtk_component_change(
            &cloned_gtk_entry_min_amount,
            &cloned_gtk_entry_transaction_type,
            &cloned_gtk_list_store_transactions,
            &cloned_gtk_entry_transaction_filter_label,
            &cloned_gtk_entry_transaction_date,
            &cloned_gtk_calendar_start_range,
            &cloned_gtk_calendar_end_range,
        );
    });
}
fn add_balance(
    balance: &BalanceData,
    gtk_label_available_overview: &gtk::Label,
    gtk_label_inmature_overview: &gtk::Label,
    gtk_label_pending_overview: &gtk::Label,
    gtk_label_total_overview: &gtk::Label,
) {
    gtk_label_available_overview.set_text(&balance.available);
    gtk_label_inmature_overview.set_text(&balance.inmature);
    gtk_label_pending_overview.set_text(&balance.pending);
    gtk_label_total_overview.set_text(&balance.total);
}
