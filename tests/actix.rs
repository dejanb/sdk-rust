mod test_data;

#[cfg(feature = "actix")]
mod tests {
    use actix_web::test;
    use actix_web::web;

    use cloudevents::binding::actix::HttpRequestExt;
    use crate::test_data::*;

    #[actix_rt::test]
    async fn test_request() {
        let mut expected = v10::minimal();
        // TODO extension is set explicitly as setting real integer value does not work properly at the moment.
        // Ideally we would just use v10::full_no_data() for this test
        expected.set_extension("someint", "10");

        let (req, payload) = test::TestRequest::post()
            .header("ce-specversion", "1.0")
            .header("ce-id", id())
            .header("ce-type", ty())
            .header("ce-source", source())
            .header("ce-someint", "10")
            .to_http_parts();

        let resp = req.to_event(web::Payload(payload)).await.unwrap();
        assert_eq!(expected, resp);
    }
}
