use super::*;
pub const SERVICE: u64 = 7;
pub fn credential() -> Secret {
    let settings = settings();
    let mut rng = CryptoRng::new(&settings.seed, settings.stream).expect("fixture seed");
    let mut random = [0; 32];
    rng.fill(&mut random);
    let mut bytes = Writer::new(40);
    bytes.put(&SERVICE.to_be_bytes()).expect("number");
    bytes.put(&random).expect("random");
    Secret::new(bytes.finish())
}
pub fn credential_digest() -> Secret {
    let words = skein_io::digest::digest(credential().expose()).0;
    let mut bytes = Writer::new(32);
    for word in words {
        bytes.put(&word.to_be_bytes()).expect("digest word");
    }
    Secret::new(bytes.finish())
}
pub fn service_kept() -> Kept {
    Kept { digest: credential_digest(), expires: Wall::from_nanos(u64::MAX) }
}
impl Exchange {
    pub fn make_service(&mut self) -> (Token, ReplyTo) {
        let (ask, expected) = make_service_case(&wire());
        let owner = self.post(keyed(w::Owner::Jig, 8, encode_ask(&ask)));
        self.authenticate();
        let Some(Request::Core(event)) = self.out.pop() else { panic!("core ask") };
        let core::Event::People(people::Event::Ask { reply_to, ask, .. }) = *event else { panic!("people ask") };
        assert_eq!(ask, expected);
        (owner, reply_to)
    }
    pub fn service_made(&mut self, to: ReplyTo) {
        self.reply(to, people::Reply::Outcome(people::Outcome::ServiceMade { person: SERVICE }));
        let Some(Request::Secrets(SecretsRequest::Get { owner })) = self.out.pop() else { panic!("service get") };
        assert_eq!(owner, SecretOwner::Service(SERVICE));
    }
    pub fn service_got(&mut self, value: Option<Kept>) {
        self.component.down(
            &self.env,
            Event::Secrets(SecretsAnswer::Got { owner: SecretOwner::Service(SERVICE), value }),
            &mut self.out,
        );
    }
    pub fn keep_service(&mut self) {
        let Some(Request::Secrets(SecretsRequest::Keep { owner, value })) = self.out.pop() else {
            panic!("service keep")
        };
        assert_eq!(owner, SecretOwner::Service(SERVICE));
        assert_eq!(value.digest, credential_digest());
        assert_eq!(value.expires, Wall::from_nanos(u64::MAX));
    }
    pub fn service_credential(&self, owner: Token) -> Secret {
        let w::Final::Done(value) = self.final_answer(owner) else { panic!("done") };
        let w::Outcome::Made(value) = value.into_parts().outcome else { panic!("made") };
        let w::Made::Service(value) = value.into_parts().what else { panic!("service") };
        let parts = value.into_parts();
        assert_eq!(parts.party, SERVICE);
        parts.credential
    }
    pub fn sign_in_service(&mut self) -> (Token, ReplyTo) {
        self.token = credential().expose().into();
        let owner = self.post_to(b"/jig/v1/sign-in/deployment", Box::new([]));
        let Some(Request::Secrets(SecretsRequest::Get { owner: who })) = self.out.pop() else {
            panic!("credential get")
        };
        assert_eq!(who, SecretOwner::Service(SERVICE));
        self.service_got(Some(service_kept()));
        let Some(Request::Core(event)) = self.out.pop() else { panic!("core sign-in") };
        let core::Event::SignIn { reply_to, identity } = *event else { panic!("sign-in") };
        assert_eq!(identity.key.provider, 2);
        assert_eq!(identity.key.subject.as_ref(), &SERVICE.to_be_bytes());
        assert!(identity.login.is_empty() && identity.name.is_empty());
        (owner, reply_to)
    }
    pub fn signed_in_answer(&self, owner: Token) -> w::SignedIn {
        let (_, bytes) = self.response(owner);
        let w::SignInAnswer::SignedIn(value) =
            w::SignInAnswer::decode(&wire(), &mut skein_lib::Reader::new(bytes)).expect("sign-in answer")
        else {
            panic!("signed in")
        };
        value
    }
}
