import org.testng.annotations.Test;


public class PubSubChannelITest extends BasePSChannelITest {

    @Test(priority = 1)
    public void createTopicExpectCorrectResponse() {
        String actual = call("channel/req_create_topic");
        validateWithFormatIdentity("channel/exp_create_topic", actual);
    }

    @Test(priority = 2)
    public void createExistentTopicExpectCorrectResponse() {
        String actual = call("channel/req_create_topic");
        validateWithFormatIdentity("channel/exp_existent_create_topic", actual);
    }

    @Test(priority = 3)
    public void postMessageExpectCorrectResponse() {
        String actual = call("channel/req_post_message");
        validateWithFormatIdentity("channel/exp_post_message", actual);
    }

    @Test(priority = 4)
    public void getMessageExpectCorrectResponse() {
        String actual = call("channel/req_get_messages");
        validateWithFormatIdentity("channel/exp_post_message", actual);
    }

    @Test(priority = 5)
    public void postMoreThanNumberAndRemoveOnGetExpectError() {
        call("channel/req_post_message");
        call("channel/req_post_message");
        String actual = call("channel/req_post_message");
        validateWithFormatIdentity("channel/exp_over_messages", actual);
    }

    @Test(priority = 6)
    public void postMoreThanLengthExpectError() {
        String actual = call("channel/req_post_long_message");
        validateWithFormatIdentity("channel/exp_long_messages", actual);
    }

    @Test(priority = 7)
    public void deleteTopicExpectCorrectResponse() {
        String actual = call("channel/req_delete_topic");
        validateWithFormatIdentity("channel/exp_delete_topic", actual);
    }

}
