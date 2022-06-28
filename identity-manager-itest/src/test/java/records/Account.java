package records;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.RequiredArgsConstructor;
import org.ic4j.candid.annotations.Field;
import org.ic4j.candid.annotations.Ignore;
import org.ic4j.candid.annotations.Name;
import org.ic4j.candid.parser.IDLType;
import org.ic4j.candid.parser.IDLValue;
import org.ic4j.candid.pojo.PojoDeserializer;
import org.ic4j.candid.types.Type;

import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;

@Builder
@AllArgsConstructor
@RequiredArgsConstructor
public class Account {
    @Field(Type.NAT64)
    public Long anchor;
    @Name("principal_id")
    public String principalId;
    public Optional<String> name;
    @Name("phone_number")
    public Optional<String> phoneNumber;
    @Ignore
    public List<Object> personas;
    @Name("access_points")
    public List<AccessPoint> accessPoints;
}


