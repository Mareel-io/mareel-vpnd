/*
 * SPDX-FileCopyrightText: 2022 Empo Inc.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use std::sync::{Arc, Mutex};

use prometheus::Registry;
use rocket::fairing::AdHoc;

use self::common::PrometheusStore;

pub(crate) mod common;
pub(crate) mod tokenauth;
mod v1;

pub(crate) struct AuthKeyProvider {
    auth_key: String,
}

pub(crate) fn stage(key: &str, registry: Arc<Mutex<Registry>>) -> AdHoc {
    let k = key.to_owned();
    AdHoc::on_ignite("API", |rocket| async {
        rocket
            .attach(v1::stage())
            .manage(AuthKeyProvider { auth_key: k })
            .manage(PrometheusStore { registry })
    })
}
