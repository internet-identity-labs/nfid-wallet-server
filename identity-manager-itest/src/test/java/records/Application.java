package records;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.RequiredArgsConstructor;
import org.ic4j.candid.annotations.Field;
import org.ic4j.candid.annotations.Name;
import org.ic4j.candid.types.Type;

@Builder
@AllArgsConstructor
@RequiredArgsConstructor
public class Application {
    public String name;
    public String domain;
    @Field(Type.NAT16)
    @Name("user_limit")
    public Short userLimit;
}
