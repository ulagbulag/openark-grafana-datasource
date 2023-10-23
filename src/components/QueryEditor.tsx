import React, { ChangeEvent } from 'react';
import { InlineField, Input } from '@grafana/ui';
import { QueryEditorProps } from '@grafana/data';
import { DataSource } from '../datasource';
import { MyDataSourceOptions, MyQuery } from '../types';

type Props = QueryEditorProps<DataSource, MyQuery, MyDataSourceOptions>;

export function QueryEditor({ query, onChange, onRunQuery }: Props) {
  const onRawQueryTextChange = (event: ChangeEvent<HTMLInputElement>) => {
    onChange({ ...query, rawQuery: event.target.value });
  };

  const { rawQuery } = query;

  return (
    <div className="gf-form">
      <InlineField label="Raw Query" labelWidth={16} tooltip="DeltaLake SQL" placeholder="SELECT * FROM my_model;">
        <Input onChange={onRawQueryTextChange} value={rawQuery || ''} />
      </InlineField>
    </div>
  );
}
