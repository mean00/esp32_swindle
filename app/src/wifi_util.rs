#![allow(dead_code)]

use esp_idf_svc::sys::{esp_wifi_get_max_tx_power, esp_wifi_set_max_tx_power};
use esp_idf_svc::sys::{
    esp_wifi_set_country, wifi_country_policy_t_WIFI_COUNTRY_POLICY_AUTO, wifi_country_t,
};

pub fn set_country_code() {
    // 4. Start Wi-Fi and Connect
    println!("Setting CC ...");
    let country = wifi_country_t {
        cc: *b"FR\0",
        schan: 1,
        nchan: 13,
        max_tx_power: 80, // 20dBm
        policy: wifi_country_policy_t_WIFI_COUNTRY_POLICY_AUTO,
    };

    unsafe {
        let err = esp_wifi_set_country(&country);
        if err != 0 {
            println!("Failed to set country code: {}", err);
        } else {
            println!("Country set to FR (Channels 1-13 enabled)");
        }
    }
}

pub fn set_wifi_tx_power() {
    unsafe {
        let mut power: i8 = 0;
        esp_wifi_get_max_tx_power(&mut power);
        println!("raw tx power {}", power);
        // 20 + 5 db
        // 30 + 8 db
        // 52 + 16 db
        // 80 + 20 db
        // --------------
        // 60 ->
        // 50 -> no
        // 40 -> no
        // 30 -> no
        esp_wifi_set_max_tx_power(45); // 34 is 8.5 dbm
        esp_wifi_get_max_tx_power(&mut power);
        println!("new tx power {}", power);
    }
}
