use async_trait::async_trait;
use mockall::mock;
use mystiko_notification::{Notification, SnsNotification};
use rusoto_core::{Region, RusotoError};
use rusoto_sns::*;

#[tokio::test]
async fn test_push() {
    let mut client = MockSnsClient::new();
    client
        .expect_publish()
        .withf(|input| {
            input.message == "Hello, world!"
                && input.subject.as_ref().unwrap() == "Test"
                && input.topic_arn.as_ref().unwrap() == "Test Topic"
        })
        .returning(|_| Ok(PublishResponse::default()));
    let notification = SnsNotification::<MockSnsClient>::builder().client(client).build();
    let message = PublishInput {
        message: "Hello, world!".to_string(),
        subject: Some("Test".to_string()),
        topic_arn: Some("Test Topic".to_string()),
        ..Default::default()
    };
    notification.push(message).await.unwrap();
}

#[tokio::test]
async fn test_from_region() {
    SnsNotification::from_region(Region::ApSoutheast1);
}

mock! {
    pub SnsClient {}

    #[async_trait]
    impl Sns for SnsClient {
        async fn add_permission(
            &self,
            input: AddPermissionInput,
        ) -> Result<(), RusotoError<AddPermissionError>>;
        async fn check_if_phone_number_is_opted_out(
            &self,
            input: CheckIfPhoneNumberIsOptedOutInput,
        ) -> Result<CheckIfPhoneNumberIsOptedOutResponse, RusotoError<CheckIfPhoneNumberIsOptedOutError>>;
        async fn confirm_subscription(
            &self,
            input: ConfirmSubscriptionInput,
        ) -> Result<ConfirmSubscriptionResponse, RusotoError<ConfirmSubscriptionError>>;
        async fn create_platform_application(
            &self,
            input: CreatePlatformApplicationInput,
        ) -> Result<CreatePlatformApplicationResponse, RusotoError<CreatePlatformApplicationError>>;
        async fn create_platform_endpoint(
            &self,
            input: CreatePlatformEndpointInput,
        ) -> Result<CreateEndpointResponse, RusotoError<CreatePlatformEndpointError>>;
        async fn create_sms_sandbox_phone_number(
            &self,
            input: CreateSMSSandboxPhoneNumberInput,
        ) -> Result<CreateSMSSandboxPhoneNumberResult, RusotoError<CreateSMSSandboxPhoneNumberError>>;
        async fn create_topic(
            &self,
            input: CreateTopicInput,
        ) -> Result<CreateTopicResponse, RusotoError<CreateTopicError>>;
        async fn delete_endpoint(
            &self,
            input: DeleteEndpointInput,
        ) -> Result<(), RusotoError<DeleteEndpointError>>;
        async fn delete_platform_application(
            &self,
            input: DeletePlatformApplicationInput,
        ) -> Result<(), RusotoError<DeletePlatformApplicationError>>;
        async fn delete_sms_sandbox_phone_number(
            &self,
            input: DeleteSMSSandboxPhoneNumberInput,
        ) -> Result<DeleteSMSSandboxPhoneNumberResult, RusotoError<DeleteSMSSandboxPhoneNumberError>>;
        async fn delete_topic(
            &self,
            input: DeleteTopicInput,
        ) -> Result<(), RusotoError<DeleteTopicError>>;
        async fn get_endpoint_attributes(
            &self,
            input: GetEndpointAttributesInput,
        ) -> Result<GetEndpointAttributesResponse, RusotoError<GetEndpointAttributesError>>;
        async fn get_platform_application_attributes(
            &self,
            input: GetPlatformApplicationAttributesInput,
        ) -> Result<
            GetPlatformApplicationAttributesResponse,
            RusotoError<GetPlatformApplicationAttributesError>,
        >;
        async fn get_sms_attributes(
            &self,
            input: GetSMSAttributesInput,
        ) -> Result<GetSMSAttributesResponse, RusotoError<GetSMSAttributesError>>;
        async fn get_sms_sandbox_account_status(
            &self,
            input: GetSMSSandboxAccountStatusInput,
        ) -> Result<GetSMSSandboxAccountStatusResult, RusotoError<GetSMSSandboxAccountStatusError>>;
        async fn get_subscription_attributes(
            &self,
            input: GetSubscriptionAttributesInput,
        ) -> Result<GetSubscriptionAttributesResponse, RusotoError<GetSubscriptionAttributesError>>;
        async fn get_topic_attributes(
            &self,
            input: GetTopicAttributesInput,
        ) -> Result<GetTopicAttributesResponse, RusotoError<GetTopicAttributesError>>;
        async fn list_endpoints_by_platform_application(
            &self,
            input: ListEndpointsByPlatformApplicationInput,
        ) -> Result<
            ListEndpointsByPlatformApplicationResponse,
            RusotoError<ListEndpointsByPlatformApplicationError>,
        >;
        async fn list_origination_numbers(
            &self,
            input: ListOriginationNumbersRequest,
        ) -> Result<ListOriginationNumbersResult, RusotoError<ListOriginationNumbersError>>;
        async fn list_phone_numbers_opted_out(
            &self,
            input: ListPhoneNumbersOptedOutInput,
        ) -> Result<ListPhoneNumbersOptedOutResponse, RusotoError<ListPhoneNumbersOptedOutError>>;
        async fn list_platform_applications(
            &self,
            input: ListPlatformApplicationsInput,
        ) -> Result<ListPlatformApplicationsResponse, RusotoError<ListPlatformApplicationsError>>;
        async fn list_sms_sandbox_phone_numbers(
            &self,
            input: ListSMSSandboxPhoneNumbersInput,
        ) -> Result<ListSMSSandboxPhoneNumbersResult, RusotoError<ListSMSSandboxPhoneNumbersError>>;
        async fn list_subscriptions(
            &self,
            input: ListSubscriptionsInput,
        ) -> Result<ListSubscriptionsResponse, RusotoError<ListSubscriptionsError>>;
        async fn list_subscriptions_by_topic(
            &self,
            input: ListSubscriptionsByTopicInput,
        ) -> Result<ListSubscriptionsByTopicResponse, RusotoError<ListSubscriptionsByTopicError>>;
        async fn list_tags_for_resource(
            &self,
            input: ListTagsForResourceRequest,
        ) -> Result<ListTagsForResourceResponse, RusotoError<ListTagsForResourceError>>;
        async fn list_topics(
            &self,
            input: ListTopicsInput,
        ) -> Result<ListTopicsResponse, RusotoError<ListTopicsError>>;
        async fn opt_in_phone_number(
            &self,
            input: OptInPhoneNumberInput,
        ) -> Result<OptInPhoneNumberResponse, RusotoError<OptInPhoneNumberError>>;
        async fn publish(
            &self,
            input: PublishInput,
        ) -> Result<PublishResponse, RusotoError<PublishError>>;
        async fn remove_permission(
            &self,
            input: RemovePermissionInput,
        ) -> Result<(), RusotoError<RemovePermissionError>>;
        async fn set_endpoint_attributes(
            &self,
            input: SetEndpointAttributesInput,
        ) -> Result<(), RusotoError<SetEndpointAttributesError>>;
        async fn set_platform_application_attributes(
            &self,
            input: SetPlatformApplicationAttributesInput,
        ) -> Result<(), RusotoError<SetPlatformApplicationAttributesError>>;
        async fn set_sms_attributes(
            &self,
            input: SetSMSAttributesInput,
        ) -> Result<SetSMSAttributesResponse, RusotoError<SetSMSAttributesError>>;
        async fn set_subscription_attributes(
            &self,
            input: SetSubscriptionAttributesInput,
        ) -> Result<(), RusotoError<SetSubscriptionAttributesError>>;
        async fn set_topic_attributes(
            &self,
            input: SetTopicAttributesInput,
        ) -> Result<(), RusotoError<SetTopicAttributesError>>;
        async fn subscribe(
            &self,
            input: SubscribeInput,
        ) -> Result<SubscribeResponse, RusotoError<SubscribeError>>;
        async fn tag_resource(
            &self,
            input: TagResourceRequest,
        ) -> Result<TagResourceResponse, RusotoError<TagResourceError>>;
        async fn unsubscribe(
            &self,
            input: UnsubscribeInput,
        ) -> Result<(), RusotoError<UnsubscribeError>>;
        async fn untag_resource(
            &self,
            input: UntagResourceRequest,
        ) -> Result<UntagResourceResponse, RusotoError<UntagResourceError>>;
        async fn verify_sms_sandbox_phone_number(
            &self,
            input: VerifySMSSandboxPhoneNumberInput,
        ) -> Result<VerifySMSSandboxPhoneNumberResult, RusotoError<VerifySMSSandboxPhoneNumberError>>;
    }
}
