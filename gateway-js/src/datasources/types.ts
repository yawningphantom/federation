import { GraphQLResponse, GraphQLRequestContext } from 'apollo-server-types';
import { Headers } from 'apollo-server-env';

export interface GraphQLDataSource<TContext extends Record<string, any> = Record<string, any>> {
  process(
    request: Pick<GraphQLRequestContext<TContext>, 'request' | 'context'>,
    headers?: Headers
  ): Promise<GraphQLResponse>;
}
