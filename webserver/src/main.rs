use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct FinancialCalc {
    interest_per_year: String,
    num_periods: String,
    present_value: String,
    payment: String,
    future_value: String,
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
            <title>Financial Calculator</title>
            <form action="/handle" method="post">
            <table padding="5">
                <tr>
                    <th>Number of Periods:</th>
                    <th>Interest per Year:</th>
                    <th>Present Value:</th>
                    <th>Payment:</th>
                    <th>Future Value:</th>
                </tr>
                <tr>
                    <td><input type="text" name="num_periods" /></td>
                    <td><input type="text" name="interest_per_year" /></td>
                    <td><input type="text" name="present_value" /></td>
                    <td><input type="text" name="payment" /></td>
                    <td><input type="text" name="future_value" /></td>
                </tr>
            </table>
            <div>
                <button type="submit">Compute</button>
            </div>
            </form>
            "#,
        )
}

fn calc_fv(n: usize, i: f64, pv: f64, pmt: f64) -> f64 {
    println!("Received - n: {n}, i: {i}, pv: {pv}, pmt: {pmt}");
    let mut fv = pv;
    for _ in 0..n {
        let int = i*fv;
        println!("pv: {fv}, pmt: {pmt}, int: {int}");
        fv = fv + pmt + int;
        println!("fv: {fv}");
    }
    fv
}

async fn handle_form(form: web::Form<FinancialCalc>) -> impl Responder {
    let payment = match form.payment.parse::<f64>() {
        Ok(val) => val,
        Err(_) => return HttpResponse::BadRequest().body("Invalid payment value"),
    };
    let periods = match form.num_periods.parse::<usize>() {
        Ok(val) => val,
        Err(_) => return HttpResponse::BadRequest().body("Invalid number of periods"),
    };
    let present_value = match form.present_value.parse::<f64>() {
        Ok(val) => val,
        Err(_) => return HttpResponse::BadRequest().body("Invalid number of periods"),
    };
    let annual_interest = match form.interest_per_year.parse::<f64>() {
        Ok(val) => val,
        Err(_) => return HttpResponse::BadRequest().body("Invalid number of periods"),
    };
    let fv = calc_fv(periods, annual_interest, present_value, payment);

    let response = format!("Future Value: ${:.2}", fv);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}