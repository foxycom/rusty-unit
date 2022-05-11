package de.unipassau.rustyunit.hir;

import static com.google.common.truth.Truth.assertThat;

import de.unipassau.rustyunit.test_case.type.traits.std.default_.Default;
import java.io.IOException;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class TyCtxtTest {

  private TyCtxt analysis;

  @BeforeEach
  void setUp() throws IOException {
    analysis = new TyCtxt(Collections.emptyList());
  }

  @Test
  void testGetGeneratorsOfOption() {
    /*List<Type> generic = List.of(new Generic("T", Collections.emptyList()));
    List<EnumVariant> variants = List.of(
        new EnumVariant("None", Collections.emptyList()),
        new EnumVariant("Some", List.of(new Param(new Generic("T", Collections.emptyList()), false, null)))
    );

    var option = new Enum("std::option::Option", generic, variants, false);
    var generators = analysis.generatorsOf(option, null);

    assertThat(generators.size()).isAtLeast(2);*/
  }

  @Test
  void testRefIsizeDoesNotImplementDefault() {
    assertThat(analysis.typesImplementing(List.of(Default.getInstance()))).isEmpty();
  }

  @Test
  void testGetGeneratorsOfRefIsizeOption() {
    /*List<EnumVariant> variants = List.of(
        new EnumVariant("None", Collections.emptyList()),
        new EnumVariant("Some", List.of(new Param(new Ref(ISize.INSTANCE), false, null)))
    );
    var option = new Enum("std::option::Option", Collections.emptyList(), variants, false);
    var generators = analysis.generatorsOf(option, null);
    System.out.println(generators);*/

  }
}