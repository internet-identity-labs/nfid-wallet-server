package proxy;

import java.util.concurrent.CompletableFuture;

import org.ic4j.agent.annotations.Argument;
import org.ic4j.agent.annotations.ResponseClass;
import org.ic4j.agent.annotations.UPDATE;
import org.ic4j.agent.annotations.Waiter;
import org.ic4j.candid.annotations.Name;
import org.ic4j.candid.types.Type;

import records.HTTPAccessPointRequest;
import records.HTTPAccountRequest;
import records.HttpResponse;

public interface IdentityManagerProxy {

  @UPDATE
  @Name("create_account")
  @Waiter(timeout = 30, sleep = 1)
  @ResponseClass(HttpResponse.class)
  public CompletableFuture<HttpResponse<?>> createAccount(@Argument(Type.RECORD) HTTPAccountRequest accountRequest);

  @UPDATE
  @Name("recover_account")
  @Waiter(timeout = 30, sleep = 1)
  @ResponseClass(HttpResponse.class)
  public CompletableFuture<HttpResponse<?>> recoverAccount(@Argument(Type.NAT64) Long anchor);

  @UPDATE
  @Name("create_access_point")
  @Waiter(timeout = 30, sleep = 1)
  @ResponseClass(HttpResponse.class)
  public CompletableFuture<HttpResponse<?>> createAccessPoint(@Argument(Type.RECORD) HTTPAccessPointRequest accessPointRequest);

}