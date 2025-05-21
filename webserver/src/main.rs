use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct MortgageForm {
    name: String,
    payment: String,
    years: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = HttpServer::new( || { 
        App::new()
            .route("/", web::get().to(get_index))
            .route("/handle", web::post().to(handle_form))
    });

    println!("Serving on http://localhost:3000...");

    server
        .bind("127.0.0.1:3000").expect("error binding server to address")
        .run().await
}

async fn get_index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(
            r#"
            <title>Simple webpage with form</title>
            <form action="/handle" method="post">
            <table padding="5">
                <tr>
                    <th>Name:</th>
                    <th>Payment:</th>
                    <th>Years:</th>
                </tr>
                <tr>
                    <td><input type="text" name="name" /></td>
                    <td><input type="text" name="payment" /></td>
                    <td><input type="text" name="years" /></td>
                </tr>
            </table>
            <div>
                <button type="submit">Compute</button>
            </div>
            </form>
            "#,
        )
}

async fn handle_form(form: web::Form<MortgageForm>) -> impl Responder {
    let payment = match form.payment.parse::<u32>() {
        Ok(val) => val,
        Err(_) => return HttpResponse::BadRequest().body("Invalid payment value"),
    };
    let years = match form.years.parse::<u32>() {
        Ok(val) => val,
        Err(_) => return HttpResponse::BadRequest().body("Invalid year value"),
    };

    let total_paid = payment as u32 * (years as u32 * 12);

    let response = format!("Name: {}. Additional Payment: {}", form.name, total_paid);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}