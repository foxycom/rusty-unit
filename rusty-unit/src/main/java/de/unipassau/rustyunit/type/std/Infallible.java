package de.unipassau.rustyunit.type.std;

import de.unipassau.rustyunit.type.AbstractEnum;
import de.unipassau.rustyunit.type.traits.std.clone.Clone;
import de.unipassau.rustyunit.type.traits.std.marker.Copy;
import java.util.Collections;
import java.util.Set;

public class Infallible extends AbstractEnum {

  public Infallible() {
    super(
        "std::convert::Infallible",
        Collections.emptyList(),
        Collections.emptyList(),
        false,
        Set.of(
            Copy.getInstance(),
            Clone.getInstance()
        )
        );
  }
}
