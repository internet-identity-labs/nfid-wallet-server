package records;

import org.ic4j.candid.annotations.Field;
import org.ic4j.candid.types.Type;

import lombok.AllArgsConstructor;

@AllArgsConstructor
public final class HTTPAccountRequest {
  
  @Field(Type.NAT64)
	public Long anchor;	

}
