mod book;
mod config;
mod external;

fn main() {
    // 설정 로드
    let config = config::load_config()
        .unwrap_or_else(|e| panic!("{}", e));

    // API 클라이언트 생성
    let client = external::nlgo::Client::new(config.api().nlgo().key().to_string());

    // 요청 생성
    let request = external::nlgo::Request::builder()
        .publisher("대원씨아이")
        .start_pub_date(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
        .end_pub_date(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap())
        .page(2)
        .size(20)
        .build()
        .unwrap();

    // API 호출 및 결과 처리
    match client.get_books(request) {
        Ok(response) => {
            println!("총 검색 결과: {} 건", response.total_count);
            println!("현재 페이지: {}", response.page_no);
            println!("\n검색된 도서 목록:");

            for (index, doc) in response.docs.iter().enumerate() {
                println!("\n도서 #{}", index + 1);
                println!("제목: {}", doc.title);
                println!("ISBN: {}", doc.ea_isbn);
                println!("출판사: {}", doc.publisher);
                println!("저자: {}", doc.author);
                println!("출판일: {}", doc.real_publish_date);
            }
        },
        Err(err) => {
            eprintln!("Error fetching books: {:?}", err);
        }
    }

}