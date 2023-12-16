mod helper;
use helper::*;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use teloxide_core::types::ParseMode;
use chrono::prelude::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting command bot...");


    //let today = Local::now().date_naive();
    //let hour_string = helper::deserialize(today)[0].realpeople;
    //println!("{}", hour_string);
    //println!("{}", helper::deserialize(NaiveDate::from_ymd_opt(2023, 12, 8).unwrap())[0].realpeople);    
    let bot = Bot::from_env();
        //.auto_send();

    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Estos son los comandos")]
enum Command {
    #[command(description = "Enseña este texto")]
    Start,
    #[command(description = "obtiene la media del aforo esperado y real de hoy")]
    AforoMedio,
    #[command(description = "obtiene la media de aforo esperado durante todo el día, especificado con el formato aaaa-mm-dd. Si no se especifica la fecha, se asumira el dia actual")]
    AforoEsperado(String),
    #[command(description = "obtiene la media de aforo medido de hoy")]
    AforoReal,
    #[command(description = "obtiene el aforo de la hora especificada en formato 24 horas. si no se especifica una hora, se asumirá que es la hora actual")]
    AforoAhora(String),
    #[command(description= "crea una gráfica del aforo esperado y real a cada hora de hoy")]
    GraficoDia,
    #[command(description= "crea una gráfica del aforo medio esperado de la siguiente semana")]
    GraficoSemana,
    #[command(description= "crea una gráfica del aforo medio esperado del siguiente mes")]
    GraficoMes,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let today = Local::now().date_naive();
    
    match cmd {
        Command::Start =>{
        println!("user @{:#?} ran command Start", msg.chat.username());
        bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?
        },
        
        Command::AforoMedio =>{
            let aforos: Vec<Aforo> = 
                match deserialize(today).await{
            
                    Ok(result) => result,
                    Err(_) => vec!( Aforo{ time: NaiveDateTime::from_timestamp_millis(0).unwrap(), realpeople: 0, expectedpeople: 0}),
                };
            
            let expectedusers = emojify_number(average_expected_users(&aforos).await).await;
            let realusers = emojify_number(average_real_users(&aforos).await).await;
            
            if aforos[0].time == NaiveDateTime::from_timestamp_millis(0).unwrap() {
        
                bot.send_message(msg.chat.id, "ha habido un problema").parse_mode(ParseMode::MarkdownV2).await?
            
            } else {
                
                bot.send_message(msg.chat.id, format!("la media de usuarios esperada es de *{}*, y la real es de *{}*", expectedusers, realusers)).parse_mode(ParseMode::MarkdownV2).await?
            
            }    
        },
    
        Command::AforoEsperado(fecha) =>{

            println!("user @{:#?} ran command AforoEsperado", msg.chat.username());
            let aforos: Vec<Aforo> = if fecha.is_empty() {
            match deserialize(today).await {
        
                Ok(result) => result,
                Err(_) => {
            
                    let date = match NaiveDate::parse_from_str(&fecha,"%Y-%m-%d"){
                        Ok(result) => result,
                        Err(_) => Local::now().date_naive(),
                    };                        
            
                    match deserialize(date).await{
            
                        Ok(result) => result,
                        Err(_) => vec!( Aforo{ time: NaiveDateTime::from_timestamp_millis(0).unwrap(), realpeople: 0, expectedpeople: 0}),
                }
                },
            }

            } else {                

                let date = match NaiveDate::parse_from_str(&fecha,"%Y-%m-%d"){
                    Ok(result) => result,
                    Err(_) => NaiveDate::from_ymd_opt(2023, 12, 23).unwrap(),
                };                        
        
                match deserialize(date).await{
        
                    Ok(result) => result,
                    Err(_) => vec!( Aforo{ time: NaiveDateTime::from_timestamp_millis(0).unwrap(), realpeople: 0, expectedpeople: 0}),
                }
            };   
            let avgusers = emojify_number(average_expected_users(&aforos).await).await;
            
            println!("{:?}", aforos[0].time);
            if aforos[0].time == NaiveDateTime::from_timestamp_millis(0).unwrap() {
        
                bot.send_message(msg.chat.id, "ha habido un problema").parse_mode(ParseMode::MarkdownV2).await?
            
            } else {            
                      
            bot.send_message(msg.chat.id, format!("la media de usuarios esperada es de *{}*", avgusers)).parse_mode(ParseMode::MarkdownV2).await?
            
            }
        },
        
        Command::AforoReal =>{
            
            println!("user @{:#?} ran command AforoReal", msg.chat.username());
            let aforos: Vec<Aforo> = 
                match deserialize(today).await{
            
                    Ok(result) => result,
                    Err(_) => vec!( Aforo{ time: NaiveDateTime::from_timestamp_millis(0).unwrap(), realpeople: 0, expectedpeople: 0}),
                };
            
            let avgusers = emojify_number(average_real_users(&aforos).await).await;

            println!("{:?}", aforos[0].time);
            if aforos[0].time == NaiveDateTime::from_timestamp_millis(0).unwrap() {
        
                bot.send_message(msg.chat.id, "ha habido un problema").parse_mode(ParseMode::MarkdownV2).await?
            
            }
            else {            
           
                bot.send_message(msg.chat.id, format!("la media de usuarios real hoy es de *{}*", avgusers)).parse_mode(ParseMode::MarkdownV2).await?
            
            }
        },
        
        Command::AforoAhora(hora) =>{


            println!("ran command AforoAhora");
            let aforos: Vec<Aforo> = 
                match deserialize(today).await{
            
                    Ok(result) => result,
                    Err(_) => vec!( Aforo{ time: NaiveDateTime::from_timestamp_millis(0).unwrap(), realpeople: 0, expectedpeople: 0}),
                };
            
            let currentusers = if hora.is_empty() {
                emojify_number(current_users(&aforos, Local::now().naive_local()).await).await
            }
            
            else {
                emojify_number(users_at_hour(&aforos, hora.parse().unwrap()).await).await
            };

            if aforos[0].time == NaiveDateTime::from_timestamp_millis(0).unwrap() {
        
                bot.send_message(msg.chat.id, "ha habido un problema").parse_mode(ParseMode::MarkdownV2).await?
            
            }
            else {
                
                bot.send_message(msg.chat.id, format!("hay *{}* personas en este momento", currentusers)).parse_mode(ParseMode::MarkdownV2).await?
            }            
        },

        Command::GraficoDia => {

            println!("ran command GraficoSemana");
            let photo = helper::histogram_today().await;
            bot.send_photo(msg.chat.id, photo).await?
        },
        
        Command::GraficoSemana => {

            println!("ran command GraficoSemana");
            let photo = helper::histogram_days(7).await;
            bot.send_photo(msg.chat.id, photo).await?
        },
        
        Command::GraficoMes => {

            println!("ran command AforoAhora");
            let photo = helper::histogram_days(30).await;
            bot.send_photo(msg.chat.id, photo).await?
        },
        
        //GraficoHoras
        //Interpolado
        //guardar chatids en archivo csv (chat.is_private > chat.username, !chat.is_private > chat.title) para mandar announcements
     };

    Ok(())
}
