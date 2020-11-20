import {
  SelectionNode,
  DocumentNode,
  FieldDefinitionNode,
  DirectiveDefinitionNode,
  ASTNode,
} from 'graphql';

export type Maybe<T> = null | undefined | T;

export type ServiceName = string | null;

export type DefaultRootOperationTypeName =
  | 'Query'
  | 'Mutation'
  | 'Subscription';

export interface ExternalFieldDefinition {
  field: FieldDefinitionNode;
  parentTypeName: string;
  serviceName: string;
}

export interface ServiceNameToKeyDirectivesMap {
  [serviceName: string]: ReadonlyArray<SelectionNode>[];
}

export interface FederationType {
  serviceName?: ServiceName;
  keys?: ServiceNameToKeyDirectivesMap;
  externals?: {
    [serviceName: string]: ExternalFieldDefinition[];
  };
  isValueType?: boolean;
  //TODO - Possibly need to add nodes to get proper node.loc that aligns with the original schema file
  nodes: {
    [serviceName: string]: ReadonlyArray<ASTNode> | ASTNode | undefined | null
  }
}

export interface FederationField {
  serviceName?: ServiceName;
  requires?: ReadonlyArray<SelectionNode>;
  provides?: ReadonlyArray<SelectionNode>;
  belongsToValueType?: boolean;

  //TODO - possibly add field, see TODO above
  field: {
    [serviceName: string]: ReadonlyArray<ASTNode> | ASTNode | undefined | null
  }
}

export interface FederationDirective {
  directiveDefinitions: {
    [serviceName: string]: DirectiveDefinitionNode;
  }
}

export interface ServiceDefinition {
  typeDefs: DocumentNode;
  name: string;
  url?: string;
}

export interface ImpactedServicesCompositionError {
  [serviceName: string] : ReadonlyArray<ASTNode> | ASTNode | undefined | null
}

declare module 'graphql/language/ast' {
  interface UnionTypeDefinitionNode {
    serviceName?: string | null;
  }
  interface UnionTypeExtensionNode {
    serviceName?: string | null;
  }

  interface EnumTypeDefinitionNode {
    serviceName?: string | null;
  }

  interface EnumTypeExtensionNode {
    serviceName?: string | null;
  }

  interface ScalarTypeDefinitionNode {
    serviceName?: string | null;
  }

  interface ScalarTypeExtensionNode {
    serviceName?: string | null;
  }

  interface ObjectTypeDefinitionNode {
    serviceName?: string | null;
  }

  interface ObjectTypeExtensionNode {
    serviceName?: string | null;
  }

  interface InterfaceTypeDefinitionNode {
    serviceName?: string | null;
  }

  interface InterfaceTypeExtensionNode {
    serviceName?: string | null;
  }

  interface InputObjectTypeDefinitionNode {
    serviceName?: string | null;
  }

  interface InputObjectTypeExtensionNode {
    serviceName?: string | null;
  }
}
