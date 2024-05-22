use actix_web::{http::Method, web, App, HttpResponse, HttpServer, Responder};
use std::fs;
use std::io::prelude::*;
use usersBD::DataBase;
use usersBD::User;
mod usersBD;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

#[derive(Deserialize, Serialize)]
struct NameFiles{
    source: String,
    dest: String
}

static mut _USERS_BD: Lazy<DataBase> = Lazy::new(|| DataBase::new());
static mut _CURR_FOLDER: String = String::new();

async fn get_handler(name_of_file: web::Path<String>) -> impl Responder {
    let mut file_read: String =unsafe { _CURR_FOLDER.clone() };
    file_read += &name_of_file;

    let mut file = match fs::File::open(file_read) {
        Ok(it) => it,
        Err(_err) => return HttpResponse::NotFound().body("Нет такого файла")
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(it) => it,
        Err(_err) => return HttpResponse::NoContent().body("Файл пуст"),
    };

    HttpResponse::Ok().body(contents)
}

async fn put_handler(name_of_file: web::Path<String>, data: web::Json<String>) -> impl Responder {
    let mut file_write: String =unsafe { _CURR_FOLDER.clone() };
    file_write += &name_of_file.clone();
    println!("{}",name_of_file);

    let mut file = match fs::File::open(file_write.clone()) {
        Ok(it) => it,
        Err(_err) => {
            match fs::File::create(file_write){
                Ok(it) => it,
                Err(_err) => return HttpResponse::NotFound().body("Нет такого файла / Не получилось создать")
            }
        }
    };

    let contents = data.clone();
    match file.write_all(contents.as_bytes()) {
        Ok(it) => it,
        Err(_err) => return HttpResponse::NotAcceptable().body("Не удалось произвести запись в файл"),
    };

    HttpResponse::Ok().body("Запись произведена")
}

async fn post_handler(name_of_file: web::Path<String>, data: web::Json<String>) -> impl Responder {
    let mut file_write: String =unsafe { _CURR_FOLDER.clone() };
    file_write += &name_of_file;

    let mut file = match fs::OpenOptions::new()
        .write(true)
        .append(true) 
        .open(file_write){
            Ok(it) => it,
            Err(_err) => return HttpResponse::NotFound().body("Нет такого файла")
        };

    let contents = data.clone();
    match file.write_all(contents.as_bytes()) {
        Ok(it) => it,
        Err(_err) => return HttpResponse::NotAcceptable().body("Не удалось произвести запись в файл"),
    };
    HttpResponse::Ok().body("Запись произведена")
}

async fn delete_handler(name_of_file: web::Path<String>) -> impl Responder {
    let mut file_delete: String =unsafe { _CURR_FOLDER.clone() };
    file_delete += &name_of_file;

    match fs::remove_file(file_delete) {
        Ok(it) => it,
        Err(_err) => return HttpResponse::Forbidden().body("Не удалось удалить файл"),
    };
    HttpResponse::Ok().body(format!("Файл {} удален", name_of_file))
}

async fn copy_handler(data: web::Json<NameFiles>) -> impl Responder {
    let files:NameFiles = data.into_inner();

    let mut file_source: String =unsafe { _CURR_FOLDER.clone() };
    file_source += &files.source.clone();
    let mut file_dest: String =unsafe { _CURR_FOLDER.clone() };
    file_dest += &files.dest.clone();

    let mut file_s = match fs::File::open(file_source) {
        Ok(it) => it,
        Err(_err) => return HttpResponse::NotFound().body(format!("Нет такого файла ({})", files.source.clone()))
    };

    let mut file_d = match fs::OpenOptions::new()
        .write(true)
        .append(true) 
        .open(file_dest){
            Ok(it) => it,
            Err(_err) => return HttpResponse::NotFound().body(format!("Нет такого файла ({})", files.dest.clone()))
        };
        
    let mut contents = String::new();
    match file_s.read_to_string(&mut contents) {
        Ok(it) => it,
        Err(_err) => return HttpResponse::NoContent().body("Файл пуст"),
    };
    match file_d.write_all(contents.as_bytes()) {
        Ok(it) => it,
        Err(_err) => return HttpResponse::NotAcceptable().body("Не удалось произвести запись в файл"),
    };

    HttpResponse::Ok().body(format!("Содержимое файла {} скопировано и добавлено в конец файла {}", files.source.clone(), files.dest.clone()))
}

async fn register_handler(data: web::Json<User>) -> impl Responder {
    let user = data.into_inner();

    let nickname = (&user).nickname.clone();
    let folder = (&user).folder.clone();
    usersBD::DataBase::add_user(unsafe { &mut _USERS_BD }, user.nickname, user.password, user.folder);
    match fs::create_dir(format!("D:\\Uni\\Labs\\http\\src\\resources\\{}", folder)){
        Ok(_it) => return HttpResponse::Ok().body(format!("Привет, {}, твоя папка в удаленном хранилище: {}", nickname, folder)),
        Err(_err) => return HttpResponse::NotAcceptable().body(format!("Не удалось создать папку: {}", _err)),
    };
}

async fn auth_handler(data: web::Json<User>) -> impl Responder {
    let user = data.into_inner();

    if usersBD::DataBase::auth_user(unsafe { &_USERS_BD }, &(&user).nickname, &(&user).password) {
        let nickname = (&user).nickname.clone();
        let folder = (&user).folder.clone();
        let mut folder_hard: String = "D:\\Uni\\Labs\\http\\src\\resources\\".to_string();
        folder_hard += &user.folder.clone();
        folder_hard += "\\";
        unsafe { _CURR_FOLDER = folder_hard.clone() }; 
        HttpResponse::Ok().body(format!("Привет {}, твоя папка в удаленном хранилище: {}", nickname, folder))
    } else {
        HttpResponse::Unauthorized().body("Иу-иу! Авторизация не пройдена")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let _users: usersBD::DataBase = DataBase::new();
    println!("Слухаю!");
    HttpServer::new(||
        App::new()
            .route("/{name_of_file}", web::get().to(get_handler))
            .route("/{name_of_file}", web::post().to(post_handler))
            .route("/{name_of_file}", web::put().to(put_handler))
            .route("/{name_of_file}", web::delete().to(delete_handler))
            .route("/", web::route().method(Method::from_bytes(b"COPY").unwrap()).to(copy_handler))
            .route("/", web::route().method(Method::from_bytes(b"NEWUSER").unwrap()).to(register_handler))
            .route("/", web::route().method(Method::from_bytes(b"USER").unwrap()).to(auth_handler))
    )
    .bind(("127.0.0.1", 402))?
    .run()
    .await
}
