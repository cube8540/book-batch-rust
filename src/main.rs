use crate::external::aladinkr;
use crate::external::error::ClientError;

mod book;
mod config;
mod external;

// 출판사별 책 정보를 가져오는 함수
fn get_books(key: &str, publisher: &str) -> Result<Vec<aladinkr::BookItem>, ClientError> {

    // 알라딘 API 클라이언트 생성
    let client = aladinkr::Client::new(key);

    // 요청 객체 생성
    let request = aladinkr::Request::builder()
        .query(publisher)  // 출판사 이름으로 검색
        .start(1)          // 첫 번째 페이지부터 시작
        .max_results(10)   // 10개 결과만 가져오기
        .build()
        .map_err(|e| ClientError::RequestFailed(format!("요청 생성 실패: {}", e)))?;

    // 검색 실행 및 결과 반환
    let response = client.get_books(request)?;
    Ok(response.items)
}

fn main() {
    let config = config::load_config().unwrap();
    // 테스트할 출판사 이름
    let publisher = "소미미디어";

    println!("'{}' 출판사의 책을 검색합니다...", publisher);

    // 책 정보 가져오기
    match get_books(config.api().aladin().key(), publisher) {
        Ok(books) => {
            println!("총 {}권의 책을 찾았습니다:", books.len());

            // 책 정보 출력
            for (i, book) in books.iter().enumerate() {
                println!("{}. {} | 저자: {} | 가격: {}원",
                         i + 1,
                         book.title,
                         book.author,
                         book.price_sales
                );
            }
        },
        Err(e) => {
            eprintln!("오류 발생: {:?}", e);
        }
    }
}
