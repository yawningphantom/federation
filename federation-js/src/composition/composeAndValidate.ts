import { composeServices } from './compose';
import {
  validateComposedSchema,
  validateServicesBeforeComposition,
  validateServicesBeforeNormalization,
} from './validate';
import { ServiceDefinition } from './types';
import { normalizeTypeDefs } from './normalize';
import { printComposedSdl } from '../service/printComposedSdl';

export function composeAndValidate(serviceList: ServiceDefinition[]) {
  const errors = validateServicesBeforeNormalization(serviceList);

  const normalizedServiceList = serviceList.map(({ name, typeDefs }) => ({
    name,
    typeDefs: normalizeTypeDefs(typeDefs),
  }));

  // generate errors or warnings of the individual services
  errors.push(...validateServicesBeforeComposition(normalizedServiceList));

  // generate a schema and any errors or warnings
  const compositionResult = composeServices(normalizedServiceList);
  errors.push(...compositionResult.errors);

  // validate the composed schema based on service information
  errors.push(
    ...validateComposedSchema({
      schema: compositionResult.schema,
      serviceList,
    }),
  );

  let typeMap = compositionResult.schema.getTypeMap();
  for (var typeName in typeMap) {
    let type = typeMap[typeName];
    if (type.astNode) {
      let fieldsToDelete = [];
      let typeAstNode = type.astNode as any;
      for (var i = 0; i < typeAstNode.fields.length; i++) {
        let field = typeAstNode.fields[i];
        if (field.directives.find((directive: any) => directive.name.value == 'internal'))
          fieldsToDelete.push(i);
      }

      fieldsToDelete.sort((a, b) => a < b ? a : b);

      for (var i = 0; i < fieldsToDelete.length; i++) {
        let indexToDelete = fieldsToDelete[i];
        typeAstNode.fields.splice(indexToDelete, 1);
      }
    }
  }

  // We shouldn't try to print the SDL if there were errors during composition
  const composedSdl =
    errors.length === 0
      ? printComposedSdl(compositionResult.schema, serviceList)
      : undefined;

  // TODO remove the warnings array once no longer used by clients
  return {
    schema: compositionResult.schema,
    warnings: [],
    errors,
    composedSdl,
  };
}
