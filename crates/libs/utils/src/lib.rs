use anyhow::anyhow;
use anyhow::Result;
use atrium_api::types::string::Did;
use serde::{Deserialize, Serialize};

pub async fn resolve_handle(handle: &str, resolver_app_view: &str) -> Result<Did> {
    let res = reqwest::get(format!(
        "{}/xrpc/com.atproto.identity.resolveHandle?handle={}",
        resolver_app_view, handle
    ))
    .await?
    .json::<ResolvedHandle>()
    .await?;

    Ok(Did::new(res.did).map_err(|e| anyhow!("Invalid DID: {}", e))?)
}

pub async fn get_did_doc(did: Did) -> Result<DidDocument> {
    // get the specific did spec
    // did:plc:abcd1e -> plc
    tracing::debug!(
        did = did.to_string(),
        method = did.method(),
        "resolving did document"
    );
    match did.method() {
        "did:plc" => {
            let res: DidDocument = reqwest::get(format!("https://plc.directory/{}", did.to_string()))
                .await?
                .error_for_status()?
                .json()
                .await?;
            Ok(res)
        }
        "did:web" => {
            let ident = did.as_str().split(':').last().unwrap();
            let res = reqwest::get(format!("https://{}/.well-known/did.json", ident))
                .await?
                .error_for_status()?
                .json()
                .await?;

            Ok(res)
        }
        _ => todo!("Identifier not supported"),
    }
}

pub fn get_pds_endpoint(doc: &DidDocument) -> Option<DidDocumentService> {
    get_service_endpoint(doc, "#atproto_pds", "AtprotoPersonalDataServer")
}

pub fn get_service_endpoint(
    doc: &DidDocument,
    svc_id: &str,
    svc_type: &str,
) -> Option<DidDocumentService> {
    doc.service
        .iter()
        .find(|svc| svc.id == svc_id && svc._type == svc_type)
        .cloned()
}

pub async fn resolve_identity(id: &str, resolver_app_view: &str) -> Result<ResolvedIdentity> {
    // is our identifier a did
    let did = if let Ok(did) = Did::new(id.to_string()) {
        did
    } else {
        // our id must be either invalid or a handle
        if let Ok(res) = resolve_handle(id, resolver_app_view).await {
            res.clone()
        } else {
            todo!("Error type for could not resolve handle")
        }
    };

    let doc = get_did_doc(did.clone()).await?;
    let pds = get_pds_endpoint(&doc);

    if pds.is_none() {
        todo!("Error for could not find PDS")
    }

    Ok(ResolvedIdentity {
        did,
        doc,
        identity: id.to_owned(),
        pds: pds.unwrap().service_endpoint,
    })
}

// want this to be reusable on case of scope expansion :(
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct ResolvedIdentity {
    pub did: Did,
    pub doc: DidDocument,
    pub identity: String,
    // should prob be url type but not really needed rn
    pub pds: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolvedHandle {
    did: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DidDocument {
    #[serde(alias = "@context")]
    pub _context: Vec<String>,
    pub id: String,
    #[serde(alias = "alsoKnownAs")]
    pub also_known_as: Vec<String>,
    #[serde(alias = "verificationMethod")]
    pub verification_method: Vec<DidDocumentVerificationMethod>,
    pub service: Vec<DidDocumentService>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DidDocumentVerificationMethod {
    pub id: String,
    #[serde(alias = "type")]
    pub _type: String,
    pub controller: String,
    #[serde(alias = "publicKeyMultibase")]
    pub public_key_multibase: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DidDocumentService {
    pub id: String,
    #[serde(alias = "type")]
    pub _type: String,
    #[serde(alias = "serviceEndpoint")]
    pub service_endpoint: String,
}
