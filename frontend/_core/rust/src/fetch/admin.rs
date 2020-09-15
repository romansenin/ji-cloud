pub mod category {
    use shared::{
        api::endpoints::{ApiEndpoint, category::*},
        domain::category::*
    };
    use crate::{
        path::api_url,
        fetch::{api_with_auth, api_with_auth_empty, FetchResult}
    };
    use uuid::Uuid;
    use wasm_bindgen::UnwrapThrowExt;

    //needs to be a function due to orphan rule
    fn category_id_from_str(id:&str) -> CategoryId {
        CategoryId(uuid_from_str(id))
    }
    //needs to be a function due to orphan rule
    fn uuid_from_str(id:&str) -> Uuid {
        Uuid::parse_str(id).unwrap_throw()
    }

    pub async fn get_all() -> FetchResult < <Get as ApiEndpoint>::Res, <Get as ApiEndpoint>::Err> {
        let req:<Get as ApiEndpoint>::Req = GetCategoryRequest {
            ids: Vec::new(), 
            scope: Some(CategoryTreeScope::Decendants)
        };
        
        let query = serde_qs::to_string(&req).unwrap_throw();

        let path = api_url(&format!("{}?{}", Get::PATH, query)); 

        api_with_auth::<_,_,()>(&path, Get::METHOD, None).await
    }

    pub async fn create(name:String, parent_id: Option<&str>) -> FetchResult < <Create as ApiEndpoint>::Res, <Create as ApiEndpoint>::Err> {

        let req:<Create as ApiEndpoint>::Req = CreateCategoryRequest {
            name,
            parent_id: parent_id.map(category_id_from_str)
        };
        api_with_auth(&api_url(Create::PATH), Create::METHOD, Some(req)).await
    }

    pub async fn rename(id:&str, name:String) -> FetchResult < <Update as ApiEndpoint>::Res, <Update as ApiEndpoint>::Err> {
        let path = Update::PATH.replace("{id}",id);
        
        let req:<Update as ApiEndpoint>::Req = UpdateCategoryRequest {
            name: Some(name),
            parent_id: None,
            index: None
        };
        api_with_auth_empty(&api_url(&path), Update::METHOD, Some(req)).await
    }

    pub async fn move_to(id:&str, index:u16) -> FetchResult < <Update as ApiEndpoint>::Res, <Update as ApiEndpoint>::Err> {
        let path = Update::PATH.replace("{id}",id);
        
        let req:<Update as ApiEndpoint>::Req = UpdateCategoryRequest {
            name: None,
            parent_id: None, 
            index: Some(index) 
        };
        api_with_auth_empty(&api_url(&path), Update::METHOD, Some(req)).await
    }

    pub async fn move_end(id:&str, parent_id:&str) -> FetchResult < <Update as ApiEndpoint>::Res, <Update as ApiEndpoint>::Err> {
        let path = Update::PATH.replace("{id}",id);
        
        let req:<Update as ApiEndpoint>::Req = UpdateCategoryRequest {
            name: None,
            parent_id: Some(Some(category_id_from_str(parent_id))),
            index: None 
        };
        api_with_auth_empty(&api_url(&path), Update::METHOD, Some(req)).await
    }

    pub async fn delete(id:&str) -> FetchResult < <Delete as ApiEndpoint>::Res, <Delete as ApiEndpoint>::Err> {
        let path = Delete::PATH.replace("{id}",id);

        api_with_auth_empty::<_,()>(&api_url(&path), Delete::METHOD, None).await
    }
}