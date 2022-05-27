package records;

import org.ic4j.candid.annotations.Field;
import org.ic4j.candid.annotations.Name;
import org.ic4j.candid.parser.IDLType;
import org.ic4j.candid.parser.IDLValue;
import org.ic4j.candid.pojo.PojoDeserializer;
import org.ic4j.candid.types.Type;

import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

public class HttpResponse<T> {
    public Optional<T[]> data;
    public Optional<String> error;
    @Name("status_code")
    @Field(Type.NAT16)
    public Short statusCode;

    public List<T> getVectorFromData(Class<T> clazz) { //todo clazz take from T
        IDLValue value = IDLValue.create(data.get());
        return Arrays.asList(value.getValue(IDLType.createType(Type.VEC)))
                .stream()
                .map(IDLValue::create)
                .map(l -> l.getValue(PojoDeserializer.create(), clazz))
                .collect(Collectors.toList());

    }
}



