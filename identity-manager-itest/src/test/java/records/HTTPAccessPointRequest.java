package records;

import org.ic4j.candid.annotations.Field;
import org.ic4j.candid.annotations.Name;
import org.ic4j.candid.types.Type;

import lombok.AllArgsConstructor;

@AllArgsConstructor
public final class HTTPAccessPointRequest {
  
  @Name("pub_key")
  @Field(Type.NAT8)
  public byte[] pubKey;

  @Name("icon")
  @Field(Type.TEXT)
	public String icon;	

  @Name("device")
  @Field(Type.TEXT)
	public String device;	

  @Name("browser")
  @Field(Type.TEXT)
	public String browser;

}
