package records;

import java.util.Optional;

import org.ic4j.candid.annotations.Field;
import org.ic4j.candid.annotations.Name;
import org.ic4j.candid.types.Type;
import org.ic4j.internetidentity.KeyType;
import org.ic4j.internetidentity.Purpose;

public final class DeviceData {
    @Name("pubkey")
    @Field(Type.NAT8)
    public byte[] pubkey;	
    @Name("alias")
    @Field(Type.TEXT)
    public String alias;
    @Name("credential_id")
    @Field(Type.VEC)
    public Optional<byte[]> credentialId;
    @Name("purpose")
    @Field(Type.VARIANT)
    public Purpose purpose; 
    @Name("key_type")
    @Field(Type.VARIANT)
    public KeyType keyType;
    @Name("protection")
    @Field(Type.VARIANT)
    public Protection protection;
}