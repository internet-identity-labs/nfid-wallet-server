package records;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.RequiredArgsConstructor;
import org.ic4j.candid.annotations.Field;
import org.ic4j.candid.annotations.Ignore;
import org.ic4j.candid.annotations.Name;
import org.ic4j.candid.types.Type;

import java.util.List;
import java.util.Optional;

@Builder
@AllArgsConstructor
@RequiredArgsConstructor
public class AccessPoint {
    @Name("principal_id")
    public String principalId;
    public String icon;
    public String device;
    public String browser;
    @Field(Type.NAT64)
    @Name("last_used")
    public Short lastUsed;
}


